use crate::{BrokerSettings, Sessions, Subscriptions};
use async_std::sync::{Arc, RwLock};

/// The main broker object
#[derive(Default, Debug, Clone)]
pub struct Broker {
    /// Settings used for this broker
    pub settings: Arc<BrokerSettings>,

    /// List of sessions
    pub sessions: Arc<RwLock<Sessions>>,

    /// List of subscriptinos
    pub subscriptions: Arc<RwLock<Subscriptions>>,
}

impl From<BrokerSettings> for Broker {
    fn from(settings: BrokerSettings) -> Self {
        assert!(settings.is_valid());
        Broker {
            settings: Arc::new(settings),
            ..Default::default()
        }
    }
}
