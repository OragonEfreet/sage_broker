use crate::BrokerSettings;
use async_std::sync::{Arc, RwLock};

/// The broker instance. Holds and the data related to broker configuration and
/// session management.
pub struct Broker {
    /// The broker configuration
    pub settings: RwLock<BrokerSettings>,
}

impl Broker {
    /// Creates a new Arc<Broker> instance by stating its configuration.
    /// The instance is not started yet and must be given to `start`.
    pub fn build(settings: BrokerSettings) -> Arc<Self> {
        Broker {
            settings: settings.into(),
        }
        .into()
    }
}
