use crate::{Broker, Peer};
use async_std::sync::Arc;
use log::error;
use sage_mqtt::{ConnAck, Packet, PingResp, ReasonCode};

mod connect;
mod subscribe;

pub async fn run(broker: Arc<Broker>, packet: Packet, peer: Arc<Peer>) {
    match packet {
        Packet::Subscribe(packet) => subscribe::run(broker, packet, peer).await,
        Packet::PingReq => peer.send(PingResp.into()).await,
        Packet::Connect(packet) => connect::run(broker, packet, peer).await,
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
