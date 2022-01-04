use crate::{BrokerSettings, Peer, Publisher, Sessions};
use sage_mqtt::{Packet, PingResp, ReasonCode};
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
) -> Result<(), ReasonCode> {
    match packet {
        Packet::Subscribe(packet) => subscribe::run(settings, packet, peer).await,
        Packet::PingReq => {
            peer.send(PingResp.into());
            Ok(())
        }
        Packet::Connect(packet) => {
            connect::run(settings, sessions, packet, peer, publisher.cache().clone()).await
        }
        Packet::Publish(packet) => publish::run(packet, sessions).await,
        _ => Err(ReasonCode::ImplementationSpecificError),
    }
}
