use crate::{Action, BrokerSettings, Control, Peer, Sessions};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use log::error;
use sage_mqtt::{ConnAck, Packet, PingReq, ReasonCode};

#[async_trait]
impl Control for Packet {
    async fn control(
        self,
        sessions: Arc<RwLock<Sessions>>,
        settings: &Arc<BrokerSettings>,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        match self {
            Packet::Subscribe(packet) => packet.control(sessions, settings, peer).await,
            Packet::PingReq => PingReq.control(sessions, settings, peer).await,
            Packet::Connect(packet) => packet.control(sessions, settings, peer).await,
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
