use crate::{Broker, Control, ControlReceiver, Peer, Session};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
};
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, PingResp, ReasonCode};

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
pub async fn control_loop(broker: Arc<Broker>, mut from_control_channel: ControlReceiver) {
    info!("Start control loop");
    while let Some(control) = from_control_channel.next().await {
        let Control(peer, packet) = control;
        debug!(
            "<<< [{}]: {:?}",
            if let Some(session) = peer.read().await.session() {
                &session.id
            } else {
                ""
            },
            packet
        );

        match treat(&broker, packet, &peer).await {
            TreatAction::Respond(packet) => peer.write().await.send(packet).await,
            TreatAction::RespondAndDisconnect(packet) => {
                peer.write().await.send_close(packet).await
            }
        };
    }
    info!("Stop control loop");
}

async fn treat(broker: &Arc<Broker>, packet: Packet, source: &Arc<RwLock<Peer>>) -> TreatAction {
    // If the broker is stopping, let's notify here the client with a
    // DISCONNECT and close the peer
    if broker.is_shutting_down().await {
        TreatAction::RespondAndDisconnect(
            Disconnect {
                reason_code: ReasonCode::ServerShuttingDown,
                ..Default::default()
            }
            .into(),
        )
    } else {
        match packet {
            Packet::Connect(packet) => treat_connect(&broker, packet, &source).await,
            Packet::PingReq => treat_pingreq(),
            _ => treat_unsupported(),
        }
    }
}

async fn treat_connect(
    broker: &Arc<Broker>,
    connect: Connect,
    peer: &Arc<RwLock<Peer>>,
) -> TreatAction {
    // First, we prepare an equivalent connack using broker policy
    // and infer the actual client_id requested for this client
    let client_id = connect.client_id.clone();
    let connack = broker.settings.read().await.acknowledge_connect(connect);
    let client_id = connack.assigned_client_id.clone().or(client_id).unwrap();

    // Whatever the connack is, it falls into two cases: Success or not.
    // TODO not always. once extended auth is available, we may send
    // something else than connack

    // Here we should attach a session to the peer
    // That means we need access to the peer.
    // TODO: We do that, no?

    if connack.reason_code == ReasonCode::Success {
        // Session creation
        let mut clients = broker.clients.write().await;

        // We search, in any other aleady existing clients, if the name is
        // already taken. If so, we remove the client from our collection
        // and disconnect the peer if any.
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

        // Now we create the new client and push it into the collection.
        let client = Arc::new(Session::new(&client_id, peer.clone()));
        clients.push(client.clone());
        info!("New client: {}", client.id);

        TreatAction::Respond(connack.into())
    } else {
        TreatAction::RespondAndDisconnect(connack.into())
    }
}

/// Simply returns a PingResp package
fn treat_pingreq() -> TreatAction {
    TreatAction::Respond(PingResp.into())
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
