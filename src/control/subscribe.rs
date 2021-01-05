use crate::{Action, BackEnd, BrokerSettings, Control, Peer};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{SubAck, Subscribe};

/// Simply returns a PingResp package
/// With the correct packet identifier
#[async_trait]
impl Control for Subscribe {
    async fn control(self, _: &BackEnd, _: &Arc<BrokerSettings>, _: &Arc<RwLock<Peer>>) -> Action {
        let suback = SubAck {
            packet_identifier: self.packet_identifier,
            ..Default::default()
        };
        Action::Respond(suback.into())
    }
}
