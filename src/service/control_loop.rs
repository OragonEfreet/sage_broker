use crate::{Broker, Client, Control, ControlReceiver, Peer};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, ReasonCode};

enum TreatAction {
    // None,
    Respond(Packet),
    RespondAndDisconnect(Packet),
    // Disconnect,
}

/// The control loop is reponsible from receiving and treating any control
/// packet. I thus represent the actual instance of a running broker.
/// The loop holds and manages the list of clients, dispatching messages from
/// client to client.
/// The loop automatically ends when all control sender channels are dropped.
/// These are held by `listen_loop` (one per peer) and the `listen_tcp`
/// tasks. Meaning when all peers are dropped and port listenning is stopped
/// The control loop ends.
pub async fn control_loop(broker: Arc<Broker>, mut control_receiver: ControlReceiver) {
    info!("Start control loop ({})", task::current().id());
    while let Some(control) = control_receiver.next().await {
        debug!("Control ({}): {}", task::current().id(), control);
        let Control(peer, packet) = control;
        match treat(&broker, packet, &peer).await {
            TreatAction::Respond(packet) => peer.write().await.send(packet).await,
            TreatAction::RespondAndDisconnect(packet) => {
                peer.write().await.send_close(packet).await
            }
            // TreatAction::Disconnect => peer.write().await.close().await,
            // _ => (),
        };
    }
    info!("Stop control loop {}", task::current().id());
}

async fn treat(broker: &Arc<Broker>, packet: Packet, source: &Arc<RwLock<Peer>>) -> TreatAction {
    debug!("{:?}", packet);
    match packet {
        Packet::Connect(packet) => treat_connect(&broker, packet, &source).await,
        _ => treat_unsupported(),
    }
}

async fn treat_connect(
    broker: &Arc<Broker>,
    connect: Connect,
    peer: &Arc<RwLock<Peer>>,
) -> TreatAction {
    let client_id = connect.client_id.clone();
    let connack = broker.settings.read().await.acknowledge_connect(connect);

    // The actual client id
    let client_id = connack.assigned_client_id.clone().or(client_id).unwrap();

    let client = {
        let mut clients = broker.clients.write().await;

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

    if connack.reason_code == ReasonCode::Success {
        TreatAction::Respond(connack.into())
    } else {
        TreatAction::RespondAndDisconnect(connack.into())
    }
}

// Dev function that will actually be deleted once all packets are supported
fn treat_unsupported() -> TreatAction {
    error!("Unsupported packet");
    TreatAction::RespondAndDisconnect(
        ConnAck {
            reason_code: ReasonCode::ImplementationSpecificError,
            ..Default::default()
        }
        .into(),
    )
}
