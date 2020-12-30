use crate::{Action, BrokerSettings, Control, Peer, SessionsBackEnd};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{PingReq, PingResp};
use std::marker::Send;

/// Simply returns a PingResp package
#[async_trait]
impl<B> Control<B> for PingReq
where
    B: SessionsBackEnd + Send,
{
    async fn control(self, _: &Arc<BrokerSettings>, _: &mut B, _: &Arc<RwLock<Peer>>) -> Action {
        Action::Respond(PingResp.into())
    }
}
