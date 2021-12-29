use crate::Cache;
use async_std::sync::Arc;
use sage_mqtt::{SubscriptionOptions, Topic};
use std::collections::HashMap;

/// The list of all subcriptions registered by the broker
#[derive(Debug, Clone)]
pub struct Subs {
    db: HashMap<Topic, (SubscriptionOptions, Option<u32>)>,
    cache: Arc<Cache>,
}

impl Subs {
    /// Builds a new Subscription DB with the given cache
    pub fn new(cache: Arc<Cache>) -> Self {
        Subs {
            db: Default::default(),
            cache,
        }
    }

    /// The number of subscriptions
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Add a subscription for the given filter to the given session
    /// Returns true if it replaces an existing one
    pub fn add(
        &mut self,
        topic: Topic,
        options: SubscriptionOptions,
        identifier: Option<u32>,
    ) -> bool {
        self.cache.clear();
        self.db.insert(topic, (options, identifier)).is_some()
    }

    /// Check wether the given session is subscribed to the given filter
    pub fn has_filter(&self, topic: &Topic) -> bool {
        self.db.contains_key(topic)
    }

    /// Check wether the given topic name matches any filter within this subs
    pub fn matches(&self, name: &Topic) -> bool {
        self.has_filter(name)
    }
}
