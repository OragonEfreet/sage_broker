use crate::{BrokerSettings, Peer, Sessions};
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
        sessions: &Arc<RwLock<Sessions>>,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action;
}
