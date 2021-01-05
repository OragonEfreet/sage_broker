use crate::{Action, BackEnd, BrokerSettings, Control, Peer};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{SubAck, Subscribe};

/// Simply returns a ConnAck package
/// With the correct packet identifier
#[async_trait]
impl Control for Subscribe {
    async fn control(
        self,
        _: &BackEnd,
        _: &Arc<BrokerSettings>,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        // Take the client if exist, from the peer, and at it a new sub
        if let Some(session) = peer.read().await.session() {
            let mut session = session.write().await;
            for (topic, _) in self.subscriptions {
                session.subscribe(&topic);
            }
        }

        let suback = SubAck {
            packet_identifier: self.packet_identifier,
            ..Default::default()
        };
        Action::Respond(suback.into())
    }
}
