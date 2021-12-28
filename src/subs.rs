use sage_mqtt::{SubscriptionOptions, Topic};
use std::collections::HashMap;

/// The list of all subcriptions registered by the broker
#[derive(Default, Debug, Clone)]
pub struct Subs {
    db: HashMap<Topic, SubscriptionOptions>,
}

impl Subs {
    /// The number of subscriptions
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Add a subscription for the given filter to the given session
    /// Returns true if it replaces an existing one
    pub fn add(&mut self, topic: Topic, options: SubscriptionOptions) -> bool {
        self.db.insert(topic, options).is_some()
    }

    /// Check wether the given session is subscribed to the given filter
    pub fn has_filter(&self, topic: Topic) -> bool {
        self.db.contains_key(&topic)
    }
}
