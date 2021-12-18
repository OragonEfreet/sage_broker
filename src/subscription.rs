use sage_mqtt::SubscriptionOptions;
use std::collections::HashMap;

/// Set of client subcriptions
#[derive(Default, Debug)]
pub struct Subscriptions(HashMap<String, SubscriptionOptions>);

impl Subscriptions {
    /// Add a new subscription, returning true if this replaces a current one.
    pub fn add(&mut self, topic_filter: &str, options: &SubscriptionOptions) -> bool {
        self.0
            .insert(topic_filter.into(), options.clone())
            .is_some()
    }

    /// Returns true if the exact topic filter is present
    pub fn has_filter(&self, topic_filter: &str) -> bool {
        self.0.contains_key(topic_filter)
    }

    /// Returns the number of subcriptions
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
