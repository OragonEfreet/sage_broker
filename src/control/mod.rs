use crate::{BrokerSettings, Peer, Publisher, Sessions};
use log::error;
use sage_mqtt::{ConnAck, Packet, PingResp, ReasonCode};
use std::sync::{Arc, RwLock};

mod connect;
mod publish;
mod subscribe;

pub async fn run(
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    packet: Packet,
    peer: Arc<Peer>,
    publisher: Arc<Publisher>,
) {
    match packet {
        Packet::Subscribe(packet) => subscribe::run(settings, packet, peer).await,
        Packet::PingReq => peer.send(PingResp.into()),
        Packet::Connect(packet) => {
            connect::run(settings, sessions, packet, peer, publisher.cache().clone()).await
        }
        Packet::Publish(packet) => publish::run(packet, sessions).await,
        _ => {
            error!("Unsupported packet: {:#?}", packet);
            peer.send_close(
                ConnAck {
                    reason_code: ReasonCode::ImplementationSpecificError,
                    ..Default::default()
                }
                .into(),
            );
        }
    }
}
