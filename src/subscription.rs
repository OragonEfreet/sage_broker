use crate::Session;
use async_std::sync::{Arc, RwLock};
use sage_mqtt::{SubscriptionOptions, Topic};

/// The list of all subcriptions registered by the broker
#[derive(Default, Debug)]
pub struct Subscriptions;

impl Subscriptions {
    /// The number of subscriptions
    pub fn len(&self) -> usize {
        0
    }

    /// Add a subscription for the given filter to the given session
    pub fn add(&mut self, _: Topic, _: Arc<RwLock<Session>>, _: SubscriptionOptions) -> bool {
        true
    }

    /// Check wether the given session is subscribed to the given filter
    pub fn has_filter(&self, _: Topic, _: Arc<RwLock<Session>>) -> bool {
        false
    }
}
