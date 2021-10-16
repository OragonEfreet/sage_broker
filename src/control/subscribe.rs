use crate::{Action, BrokerSettings, Control, Peer, Sessions};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{ReasonCode, SubAck, Subscribe};

/// Simply returns a ConnAck package
/// With the correct packet identifier
#[async_trait]
impl Control for Subscribe {
    async fn control(
        self,
        _: Arc<RwLock<Sessions>>,
        _: Arc<BrokerSettings>,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        let mut suback = SubAck {
            packet_identifier: self.packet_identifier,
            ..Default::default()
        };

        // Take the client if exist, from the peer, and at it a new sub
        if let Some(session) = peer.read().await.session() {
            let mut session = session.write().await;
            for (topic, _) in self.subscriptions {
                session.subscribe(&topic);
                suback.reason_codes.push(ReasonCode::Success);
            }
        }

        Action::Respond(suback.into())
    }
}
