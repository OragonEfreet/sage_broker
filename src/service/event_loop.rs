use crate::{Broker, Client, Event, EventReceiver, Peer};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, ReasonCode};

struct LoopData {
    config: Arc<RwLock<Broker>>,
    clients: RwLock<Vec<Arc<Client>>>,
}

// An event loop is made for each started broker
// The event_loop is responsible for maintaining most data related to
// the broker, such as the list of clients.
pub async fn event_loop(config: Arc<RwLock<Broker>>, event_receiver: EventReceiver) {
    LoopData {
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
