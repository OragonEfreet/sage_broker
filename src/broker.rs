use crate::{Sessions, Subscriptions};
use async_std::sync::RwLock;

/// The main broker object
#[derive(Default, Debug)]
pub struct Broker {
    /// List of sessions
    pub sessions: RwLock<Sessions>,

    /// List of subscriptinos
    pub subscriptions: RwLock<Subscriptions>,
}
