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
