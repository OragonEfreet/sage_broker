use crate::{Action, BackEnd, BrokerSettings, Control, Peer};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{PingReq, PingResp};

/// Simply returns a PingResp package
#[async_trait]
impl Control for PingReq {
    async fn control(self, _: &BackEnd, _: &Arc<BrokerSettings>, _: &Arc<RwLock<Peer>>) -> Action {
        Action::Respond(PingResp.into())
    }
}
