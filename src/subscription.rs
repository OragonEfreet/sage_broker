use std::collections::HashSet;

/// Description for a specific session subscription
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Subscription {
    /// The topic the session subscribes to
    pub topic_filter: String,
}

impl From<&str> for Subscription {
    fn from(value: &str) -> Self {
        Subscription {
            topic_filter: value.into(),
        }
    }
}

/// Set of client subcriptions
#[derive(Default, Debug)]
pub struct Subscriptions(HashSet<Subscription>);

impl Subscriptions {
    /// Add a new subscription, returning true if this replaces a current one.
    pub fn add(&mut self, topic_filter: &str) -> bool {
        !self.0.insert(topic_filter.into())
    }

    /// Returns true if the exact topic filter is present
    pub fn has_filter(&self, topic_filter: &str) -> bool {
        self.0.contains(&topic_filter.into())
    }

    /// Returns the number of subcriptions
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
