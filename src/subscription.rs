use crate::Session;
use sage_mqtt::{SubscriptionOptions, Topic};
use std::collections::HashMap;

/// Set of client subcriptions
#[derive(Default, Debug)]
pub struct ToDropSubscriptions(HashMap<Topic, SubscriptionOptions>);

impl ToDropSubscriptions {
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

/// The list of all subcriptions registered by the broker
#[derive(Default, Debug)]
pub struct Subscriptions;

impl Subscriptions {
    /// The number of subscriptions
    pub fn len(&self) -> usize {
        0
    }

    /// Add a subscription for the given filter to the given session
    pub fn add(&mut self, _: Topic, _: Session, _: SubscriptionOptions) -> bool {
        true
    }

    /// Check wether the given session is subscribed to the given filter
    pub fn has_filter(_: Topic, _: Session) -> bool {
        false
    }
}
