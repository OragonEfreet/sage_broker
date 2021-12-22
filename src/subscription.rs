use sage_mqtt::SubscriptionOptions;
use sage_mqtt::Topic;
use std::collections::HashMap;

/// Set of client subcriptions
#[derive(Default, Debug)]
pub struct Subscriptions(HashMap<Topic, SubscriptionOptions>);

impl Subscriptions {
    /// Add a new subscription, returning true if this replaces a current one.
    pub fn add(&mut self, filter: Topic, options: &SubscriptionOptions) -> bool {
        self.0.insert(filter, options.clone()).is_some()
    }

    /// Returns true if the exact topic filter is present
    pub fn has_filter(&self, filter: &Topic) -> bool {
        self.0.contains_key(filter)
    }

    /// Returns the number of subcriptions
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
