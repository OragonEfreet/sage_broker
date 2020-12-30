use crate::{BrokerSettings, Peer, Session, SessionsBackEnd, Trigger};
use async_std::sync::{Arc, RwLock};
use log::{debug, error};
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, PingResp, ReasonCode, SubAck, Subscribe};

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
        "[{:?}] <<< {:?}",
        if let Some(s) = source.read().await.session() {
            s.read().await.client_id().into()
        } else {
            String::from("N/A")
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
            Packet::Subscribe(packet) => treat_subscribe(packet).await,
            Packet::Connect(packet) => treat_connect(&settings, sessions, packet, &source).await,
            Packet::PingReq => treat_pingreq(),
            _ => treat_unsupported(),
        }
    }
}

// Manages the creation of a new session, possibly taking over a new one
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
    let mut connack = settings.acknowledge_connect(&connect);

    if connack.reason_code == ReasonCode::Success {
        let client_id = connack
            .assigned_client_id
            .clone()
            .or(connect.client_id)
            .unwrap();

        let clean_start = connect.clean_start;
        // Session creation/overtaking
        // First, we get the may be existing session from the db:
        let session = {
            if let Some(session) = sessions.take(&client_id).await {
                // If the existing session has a peer, it'll be disconnected with takeover
                if let Some(peer) = session.read().await.peer() {
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

                if clean_start {
                    connack.session_present = false;
                    let client_id = client_id.clone();
                    Arc::new(RwLock::new(Session::new(&client_id, peer)))
                } else {
                    connack.session_present = true;
                    session.write().await.set_peer(peer);
                    session
                }
            } else {
                connack.session_present = false;
                let client_id = client_id.clone();
                Arc::new(RwLock::new(Session::new(&client_id, peer)))
            }
        };
        sessions.add(session.clone()).await;
        peer.write().await.bind(session);

        TreatAction::Respond(connack.into())
    } else {
        TreatAction::RespondAndDisconnect(connack.into())
    }
}

/// Simply returns a PingResp package
/// With the correct packet identifier
async fn treat_subscribe(packet: Subscribe) -> TreatAction {
    let suback = SubAck {
        packet_identifier: packet.packet_identifier,
        ..Default::default()
    };
    TreatAction::Respond(suback.into())
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
