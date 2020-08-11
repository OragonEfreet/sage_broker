use crate::{
    service::{self, send_loop},
    Broker, Client, Event, EventReceiver, EventSender, Peer,
};
use async_std::{
    net::TcpStream,
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, ReasonCode};

struct LoopData {
    event_sender: EventSender,
    config: Arc<RwLock<Broker>>,
    clients: RwLock<Vec<Arc<Client>>>,
}

// An event loop is made for each started broker
// The event_loop is responsible for maintaining most data related to
// the broker, such as the list of clients.
pub async fn event_loop(
    config: Arc<RwLock<Broker>>,
    event_sender: EventSender,
    event_receiver: EventReceiver,
) {
    LoopData {
        event_sender,
        config,
        clients: Default::default(),
    }
    .start(event_receiver)
    .await;
}

impl LoopData {
    async fn start(&self, mut event_receiver: EventReceiver) {
        info!("Start event loop ({})", task::current().id());
        while let Some(event) = event_receiver.next().await {
            debug!("Event ({}): {}", task::current().id(), event);
            match event {
                Event::EndPeer(_) => debug!("End peer"),
                Event::NewPeer(stream) => self.create_peer(stream).await,
                Event::Control(peer, packet) => {
                    match self.treat(packet, &peer).await {
                        (false, Some(packet)) => peer.write().await.send(packet).await,
                        (true, None) => peer.write().await.close().await,
                        (true, Some(packet)) => peer.write().await.send_close(packet).await,
                        _ => (),
                    };
                }
            }
        }
        info!("Stop event loop {}", task::current().id());
    }

    async fn treat(&self, packet: Packet, source: &Arc<RwLock<Peer>>) -> (bool, Option<Packet>) {
        debug!("{:?}", packet);
        match packet {
            Packet::Connect(packet) => self.treat_connect(packet, &source).await,
            _ => treat_unsupported(),
        }
    }

    async fn treat_connect(
        &self,
        connect: Connect,
        peer: &Arc<RwLock<Peer>>,
    ) -> (bool, Option<Packet>) {
        let client_id = connect.client_id.clone();
        let connack = self.config.read().await.acknowledge_connect(connect);

        // The actual client id
        let client_id = connack.assigned_client_id.clone().or(client_id).unwrap();

        let client = {
            let mut clients = self.clients.write().await;

            if let Some(index) = clients.iter().position(|c| c.id == client_id) {
                let client = clients.swap_remove(index);
                if let Some(peer) = client.peer.upgrade() {
                    let packet = Disconnect {
                        reason_code: ReasonCode::SessionTakenOver,
                        ..Default::default()
                    };
                    peer.write().await.send_close(packet.into()).await;
                }
            }

            let client = Arc::new(Client::new(&client_id, peer.clone()));
            clients.push(client.clone());
            client
        };

        debug!("New client: {}", client.id);

        // Here we should attach a client to the peer
        // That means we need access to the peer.

        (
            connack.reason_code != ReasonCode::Success,
            Some(connack.into()),
        )
    }

    // Creation of a new peer involves a new `Peer` instance along with starting
    // an async loops for receiving (`listen_loop`) and sending (`sending_loop`)
    // packets.
    async fn create_peer(&self, stream: TcpStream) {
        match stream.peer_addr() {
            Err(e) => error!("Cannot get peer addr: {:?}", e),
            Ok(_) => {
                // New peer (no client for now)
                // Create the packet send/receive channel
                // Launch the sender loop
                // Create peer
                // Launch the listen peer loop
                let stream = Arc::new(stream);

                let (packet_sender, packet_receiver) = mpsc::unbounded();
                let sender_handle = task::spawn(send_loop(packet_receiver, stream.clone()));
                let peer = Peer::new(packet_sender, sender_handle);
                let peer = Arc::new(RwLock::new(peer));

                // Start the connection loop for this stream
                task::spawn(service::listen_loop(
                    peer,
                    self.event_sender.clone(),
                    self.config.read().await.keep_alive,
                    stream,
                ));
            }
        };
    }
}

// Dev function that will actually be deleted once all packets are supported
fn treat_unsupported() -> (bool, Option<Packet>) {
    error!("Unsupported packet");
    (
        true,
        Some(
            ConnAck {
                reason_code: ReasonCode::ImplementationSpecificError,
                ..Default::default()
            }
            .into(),
        ),
    )
}
