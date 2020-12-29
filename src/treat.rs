use crate::{BrokerSettings, Peer, Session, SessionsBackEnd, Trigger};
use async_std::sync::{Arc, RwLock};
use log::{debug, error};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, PingResp, ReasonCode};

pub enum TreatAction {
    // None,
    Respond(Packet),
    RespondAndDisconnect(Packet),
    // Disconnect,
}

pub async fn treat<B>(
    settings: &Arc<BrokerSettings>,
    sessions: &mut B,
    packet: Packet,
    source: &Arc<RwLock<Peer>>,
    shutdown: &Trigger,
) -> TreatAction
where
    B: SessionsBackEnd,
{
    debug!(
        "<<< [{}]: {:?}",
        if let Some(session) = source.read().await.session() {
            session.client_id()
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

async fn treat_connect<B>(
    settings: &Arc<BrokerSettings>,
    sessions: &mut B,
    connect: Connect,
    peer: &Arc<RwLock<Peer>>,
) -> TreatAction
where
    B: SessionsBackEnd,
{
    // First, we prepare an first connack using broker policy
    // and infer the actual client_id requested for this client
    let connack = settings.acknowledge_connect(&connect);

    if connack.reason_code == ReasonCode::Success {
        let client_id = connack
            .assigned_client_id
            .clone()
            .or(connect.client_id)
            .unwrap();
        // Session creation/overtaking
        // First, we get the may be existing session from the db:
        let session = {
            if let Some(session) = sessions.take(&client_id).await {
                {
                    let mut session = session.write().await; // We take the session for writing
                                                             // If the session already has a peer, we will notify them
                    if let Some(peer) = session.peer() {
                        peer.write()
                            .await
                            .send_close(
                                Disconnect {
                                    reason_code: ReasonCode::SessionTakenOver,
                                    ..Default::default()
                                }
                                .into(),
                            )
                            .await;
                    }
                    session.set_peer(peer);
                }
                session
            } else {
                let client_id = client_id.clone();
                Arc::new(RwLock::new(Session::new(&client_id, peer)))
            }
        };
        sessions.add(session).await;

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
