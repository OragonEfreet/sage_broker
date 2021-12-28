use crate::{BrokerSettings, Sessions, Subscriptions};
use async_std::sync::RwLock;

/// The main broker object
#[derive(Default, Debug)]
pub struct Broker {
    /// Settings used for this broker
    pub settings: BrokerSettings,

    /// List of sessions
    pub sessions: RwLock<Sessions>,

    /// List of subscriptinos
    pub subscriptions: RwLock<Subscriptions>,
}

impl From<BrokerSettings> for Broker {
    fn from(settings: BrokerSettings) -> Self {
        assert!(settings.is_valid());
        Broker {
            settings,
            ..Default::default()
        }
    }
}
