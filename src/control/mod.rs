use crate::{BrokerSettings, Peer, Sessions, Subscriptions};
use async_std::sync::{Arc, RwLock};
use log::error;
use sage_mqtt::{ConnAck, Packet, PingResp, ReasonCode};

mod connect;
mod subscribe;

pub async fn run(
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    subscriptions: Arc<RwLock<Subscriptions>>,
    packet: Packet,
    peer: Arc<Peer>,
) {
    match packet {
        Packet::Subscribe(packet) => subscribe::run(settings, subscriptions, packet, peer).await,
        Packet::PingReq => peer.send(PingResp.into()).await,
        Packet::Connect(packet) => connect::run(settings, sessions, packet, peer).await,
        _ => {
            error!("Unsupported packet: {:#?}", packet);
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
