use crate::{BrokerSettings, Peer, SessionsBackEnd};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::Packet;
use std::marker::Send;

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
pub trait Control<B>
where
    B: SessionsBackEnd + Send,
{
    async fn control(
        self,
        settings: &Arc<BrokerSettings>,
        sessions: &mut B,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action;
}
