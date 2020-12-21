use crate::{BrokerSettings, Peer, Session, Trigger};
use async_std::sync::{Arc, RwLock};
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, PingResp, ReasonCode};

pub enum TreatAction {
    // None,
    Respond(Packet),
    RespondAndDisconnect(Packet),
    // Disconnect,
}

pub async fn treat(
    settings: &Arc<BrokerSettings>,
    sessions: &mut Vec<Arc<Session>>,
    packet: Packet,
    source: &Arc<RwLock<Peer>>,
    shutdown: &Trigger,
) -> TreatAction {
    debug!(
        "<<< [{}]: {:?}",
        if let Some(session) = source.read().await.session() {
            &session.id
        } else {
            ""
        },
        packet
    );
    // If the broker is stopping, let's notify here the client with a
    // DISCONNECT and close the peer
    if shutdown.is_fired().await {
        TreatAction::RespondAndDisconnect(
            Disconnect {
                reason_code: ReasonCode::ServerShuttingDown,
                ..Default::default()
            }
            .into(),
        )
    } else {
        match packet {
            Packet::Connect(packet) => treat_connect(&settings, sessions, packet, &source).await,
            Packet::PingReq => treat_pingreq(),
            _ => treat_unsupported(),
        }
    }
}

async fn treat_connect(
    settings: &Arc<BrokerSettings>,
    sessions: &mut Vec<Arc<Session>>,
    connect: Connect,
    peer: &Arc<RwLock<Peer>>,
) -> TreatAction {
    // First, we prepare an first connack using broker policy
    // and infer the actual client_id requested for this client
    let client_id = connect.client_id.clone();
    let connack = settings.acknowledge_connect(&connect);
    let client_id = connack.assigned_client_id.clone().or(client_id).unwrap();

    if connack.reason_code == ReasonCode::Success {
        // Session creation/overtaking

        // We search, in any other aleady existing sessions, if the name is
        // already taken. If so, we extract the client.
        let client = {
            // If a session exists for the same client id
            if let Some(index) = sessions.iter().position(|c| c.id == client_id) {
                let client = sessions.swap_remove(index); // We take it

                // If the session already has a peer
                if let Some(peer) = client.peer.upgrade() {
                    let packet = Disconnect {
                        reason_code: ReasonCode::SessionTakenOver,
                        ..Default::default()
                    };
                    peer.write().await.send_close(packet.into()).await;
                }
                client
            } else {
                Arc::new(Session::new(&client_id, peer.clone()))
            }
        };

        // Now we create the new client and push it into the collection.
        sessions.push(client.clone());
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
