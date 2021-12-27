use crate::Session;
use async_std::sync::{Arc, RwLock};
use sage_mqtt::{SubscriptionOptions, Topic};
use std::collections::HashMap;

/// The list of all subcriptions registered by the broker
#[derive(Default, Debug)]
pub struct Subscriptions {
    db: HashMap<(Topic, String), SubscriptionOptions>,
}

impl Subscriptions {
    /// The number of subscriptions
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Add a subscription for the given filter to the given session
    /// Returns true if it replaces an existing one
    pub async fn add(
        &mut self,
        topic: Topic,
        session: Arc<RwLock<Session>>,
        options: SubscriptionOptions,
    ) -> bool {
        self.db
            .insert((topic, session.read().await.id().into()), options)
            .is_some()
    }

    /// Check wether the given session is subscribed to the given filter
    pub async fn has_filter(&self, topic: Topic, session: Arc<RwLock<Session>>) -> bool {
        self.db
            .contains_key(&(topic, String::from(session.read().await.id())))
    }
}
