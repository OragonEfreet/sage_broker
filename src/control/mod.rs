use crate::{BrokerSettings, Peer, Sessions};
use async_std::sync::{Arc, RwLock};
use log::error;
use sage_mqtt::{ConnAck, Packet, PingResp, ReasonCode, SubAck, Subscribe};

mod connect;

pub async fn packet(
    packet: Packet,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: Arc<Peer>,
) {
    match packet {
        Packet::Subscribe(packet) => control_subscribe(packet, peer).await,
        Packet::PingReq => peer.send(PingResp.into()).await,
        Packet::Connect(packet) => connect::run(packet, sessions, settings, peer).await,
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
