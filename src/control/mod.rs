use crate::{BrokerSettings, Peer, Sessions};
use async_std::sync::{Arc, RwLock};
use log::error;
use sage_mqtt::{ConnAck, Packet, PingResp, ReasonCode};

mod connect;
mod subscribe;

pub async fn run(
    packet: Packet,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: Arc<Peer>,
) {
    match packet {
        Packet::Subscribe(packet) => subscribe::run(packet, settings, peer).await,
        Packet::PingReq => peer.send(PingResp.into()).await,
        Packet::Connect(packet) => connect::run(packet, sessions, settings, peer).await,
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
