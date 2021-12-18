use crate::{BrokerSettings, Peer, Session, Sessions};
use async_std::sync::{Arc, RwLock};
use log::error;
use sage_mqtt::{ConnAck, Connect, Disconnect, Packet, PingResp, ReasonCode, SubAck, Subscribe};

pub async fn packet(
    packet: Packet,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: Arc<Peer>,
) {
    match packet {
        Packet::Subscribe(packet) => control_subscribe(packet, peer).await,
        Packet::PingReq => peer.send(PingResp.into()).await,
        Packet::Connect(packet) => control_connect(packet, sessions, settings, peer).await,
        _ => {
            error!("Unsupported packet: {:?}", packet);
            peer.send_close(
                ConnAck {
                    reason_code: ReasonCode::ImplementationSpecificError,
                    ..Default::default()
                }
                .into(),
            )
            .await;
        }
    }
}

async fn control_connect(
    connect: Connect,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: Arc<Peer>,
) {
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
        // TODO: This can be simplified
        let session = {
            if let Some(session) = sessions.take(&client_id) {
                // If the existing session has a peer, it'll be disconnected with takeover
                if let Some(peer) = session.read().await.peer() {
                    peer.send_close(
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
                    Arc::new(RwLock::new(Session::new(&client_id, peer.clone())))
                } else {
                    connack.session_present = true;
                    session.write().await.set_peer(peer.clone());
                    session
                }
            } else {
                connack.session_present = false;
                let client_id = client_id.clone();
                Arc::new(RwLock::new(Session::new(&client_id, peer.clone())))
            }
        };
        sessions.add(session.clone());
        peer.bind(session).await;
        peer.send(connack.into()).await;
    } else {
        peer.send_close(connack.into()).await;
    }
}

/// Simply returns a ConnAck package
/// With the correct packet identifier
async fn control_subscribe(packet: Subscribe, peer: Arc<Peer>) {
    let mut suback = SubAck {
        packet_identifier: packet.packet_identifier,
        ..Default::default()
    };

    // Take the client if exist, from the peer, and at it a new sub
    if let Some(session) = peer.session().await {
        let mut session = session.write().await;
        for (topic, options) in packet.subscriptions {
            session.subscribe(&topic, &options);
            suback.reason_codes.push(ReasonCode::Success);
        }
    }

    peer.send(suback.into()).await
}
