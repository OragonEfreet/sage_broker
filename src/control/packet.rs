use crate::{Action, BackEnd, BrokerSettings, Control, Peer};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use log::error;
use sage_mqtt::{ConnAck, Packet, PingReq, ReasonCode};

#[async_trait]
impl Control for Packet {
    async fn control(
        self,
        backend: &BackEnd,
        settings: &Arc<BrokerSettings>,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        match self {
            Packet::Subscribe(packet) => packet.control(backend, settings, peer).await,
            Packet::PingReq => PingReq.control(backend, settings, peer).await,
            Packet::Connect(packet) => packet.control(backend, settings, peer).await,
            _ => {
                error!("Unsupported packet: {:?}", self);
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
}
