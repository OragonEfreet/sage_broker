use crate::{Action, BrokerSettings, Control, Peer, SessionsBackEnd};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use log::error;
use sage_mqtt::{ConnAck, Packet, PingReq, ReasonCode};
use std::marker::Send;

#[async_trait]
impl<B> Control<B> for Packet
where
    B: SessionsBackEnd + Send,
{
    async fn control(
        self,
        settings: &Arc<BrokerSettings>,
        sessions: &mut B,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        match self {
            Packet::Subscribe(packet) => packet.control(settings, sessions, peer).await,
            Packet::PingReq => PingReq.control(settings, sessions, peer).await,
            Packet::Connect(packet) => packet.control(settings, sessions, peer).await,
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
