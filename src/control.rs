use crate::{BrokerSettings, Peer, Session, Sessions};
use async_std::sync::{Arc, RwLock};
use log::error;
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, PingResp, ReasonCode, SubAck, Subscribe};

pub enum Action {
    Respond(Packet),
    RespondAndDisconnect(Packet),
}

pub async fn packet(
    packet: Packet,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: &Arc<RwLock<Peer>>,
) -> Action {
    match packet {
        Packet::Subscribe(packet) => control_subscribe(packet, peer).await,
        Packet::PingReq => Action::Respond(PingResp.into()),
        Packet::Connect(packet) => control_connect(packet, sessions, settings, peer).await,
        _ => {
            error!("Unsupported packet: {:?}", packet);
            Action::RespondAndDisconnect(
                ConnAck {
                    reason_code: ReasonCode::ImplementationSpecificError,
                    ..Default::default()
                }
                .into(),
            )
        }
    }
}

async fn control_connect(
    connect: Connect,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: &Arc<RwLock<Peer>>,
) -> Action {
    // First, we prepare an first connack using broker policy
    // and infer the actual client_id requested for this client
    let mut connack = settings.acknowledge_connect(&connect);

    if connack.reason_code == ReasonCode::Success {
        let client_id = connack
            .assigned_client_id
            .clone()
            .or(connect.client_id)
            .unwrap();

        let mut sessions = sessions.write().await;

        let clean_start = connect.clean_start;
        // Session creation/overtaking
        // First, we get the may be existing session from the db:
        let session = {
            if let Some(session) = sessions.take(&client_id) {
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
        sessions.add(session.clone());
        peer.write().await.bind(session);

        Action::Respond(connack.into())
    } else {
        Action::RespondAndDisconnect(connack.into())
    }
}

/// Simply returns a ConnAck package
/// With the correct packet identifier
async fn control_subscribe(packet: Subscribe, peer: &Arc<RwLock<Peer>>) -> Action {
    let mut suback = SubAck {
        packet_identifier: packet.packet_identifier,
        ..Default::default()
    };

    // Take the client if exist, from the peer, and at it a new sub
    if let Some(session) = peer.read().await.session() {
        let mut session = session.write().await;
        for (topic, _) in packet.subscriptions {
            session.subscribe(&topic);
            suback.reason_codes.push(ReasonCode::Success);
        }
    }

    Action::Respond(suback.into())
}
