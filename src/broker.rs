use crate::Subscriptions;
use async_std::sync::RwLock;

/// The main broker object
#[derive(Default, Debug)]
pub struct Broker {
    /// List of subscriptinos
    pub subscriptions: RwLock<Subscriptions>,
}
