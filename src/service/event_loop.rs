use crate::{
    service::{self, send_loop},
    Broker, Event, EventReceiver, EventSender, Peer,
};
use async_std::{
    net::TcpStream,
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Connect, Packet, ReasonCode};

struct LoopData {
    event_sender: EventSender,
    config: Arc<Broker>,
}

// An event loop is made for each started broker
// The event_loop is responsible for maintaining most data related to
// the broker, such as the list of clients.
pub async fn event_loop(
    config: Arc<Broker>,
    event_sender: EventSender,
    event_receiver: EventReceiver,
) {
    LoopData {
        event_sender,
        config,
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
                    match self.treat(packet).await {
                        (false, Some(packet)) => peer.write().await.send(packet).await,
                        (true, maybe_packet) => peer.write().await.close(maybe_packet).await,
                        _ => (),
                    };
                }
            }
        }
        info!("Stop event loop {}", task::current().id());
    }

    async fn treat(&self, packet: Packet) -> (bool, Option<Packet>) {
        debug!("{:?}", packet);
        match packet {
            Packet::Connect(packet) => self.treat_connect(packet).await,
            _ => treat_unsupported(),
        }
    }

    async fn treat_connect(&self, connect: Connect) -> (bool, Option<Packet>) {
        let packet = self.config.acknowledge_connect(connect);
        // source.write().await.send(packet.into()).await;
        (false, Some(packet.into()))
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
                    self.config.keep_alive,
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
