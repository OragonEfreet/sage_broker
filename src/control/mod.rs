use crate::{BackEnd, BrokerSettings, Peer};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::Packet;

mod connect;
mod packet;
mod pingreq;
mod subscribe;

pub enum Action {
    //    None,
    Respond(Packet),
    RespondAndDisconnect(Packet),
    // Disconnect,
}

#[async_trait]
pub trait Control {
    async fn control(
        self,
        settings: &Arc<BrokerSettings>,
        backen: &BackEnd,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action;
}
