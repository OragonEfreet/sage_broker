use crate::{Action, BrokerSettings, Control, Peer, SessionsBackEnd};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{SubAck, Subscribe};
use std::marker::Send;

/// Simply returns a PingResp package
/// With the correct packet identifier
#[async_trait]
impl<B> Control<B> for Subscribe
where
    B: SessionsBackEnd + Send,
{
    async fn control(self, _: &Arc<BrokerSettings>, _: &mut B, _: &Arc<RwLock<Peer>>) -> Action {
        let suback = SubAck {
            packet_identifier: self.packet_identifier,
            ..Default::default()
        };
        Action::Respond(suback.into())
    }
}
