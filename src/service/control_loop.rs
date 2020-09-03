use crate::{Broker, Client, Control, ControlReceiver, Peer};
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

/// The control loop is reponsible from receiving and treating any control
/// packet. I thus represent the actual instance of a running broker.
/// The loop holds and manages the list of clients, dispatching messages from
/// client to client.
/// The loop automatically ends when all control sender channels are dropped.
/// These are held by `listen_loop` (one per peer) and the `listen_tcp`
/// tasks. Meaning when all peers are dropped and port listenning is stopped
/// The control loop ends.
pub async fn control_loop(config: Arc<RwLock<Broker>>, control_receiver: ControlReceiver) {
    LoopData {
        config,
        clients: Default::default(),
    }
    .start(control_receiver)
    .await;
}

impl LoopData {
    async fn start(&self, mut control_receiver: ControlReceiver) {
        info!("Start control loop ({})", task::current().id());
        while let Some(control) = control_receiver.next().await {
            debug!("Control ({}): {}", task::current().id(), control);
            let Control(peer, packet) = control;
            match self.treat(packet, &peer).await {
                (false, Some(packet)) => peer.write().await.send(packet).await,
                (true, None) => peer.write().await.close().await,
                (true, Some(packet)) => peer.write().await.send_close(packet).await,
                _ => (),
            };
        }
        info!("Stop control loop {}", task::current().id());
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
