use crate::{Action, BackEnd, BrokerSettings, Control, Peer};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use log::error;
use sage_mqtt::{ConnAck, Packet, PingReq, ReasonCode};

#[async_trait]
impl Control for Packet {
    async fn control(
        self,
        settings: &Arc<BrokerSettings>,
        backend: &BackEnd,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        match self {
            Packet::Subscribe(packet) => packet.control(settings, backend, peer).await,
            Packet::PingReq => PingReq.control(settings, backend, peer).await,
            Packet::Connect(packet) => packet.control(settings, backend, peer).await,
            _ => {
                error!("Unsupported packet");
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
