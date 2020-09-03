use crate::{BrokerSettings, Client};
use async_std::sync::{Arc, RwLock};

/// The broker instance. Holds and the data related to broker configuration and
/// clients management.
pub struct Broker {
    /// The broker configuration
    pub settings: RwLock<BrokerSettings>,
    pub(crate) clients: RwLock<Vec<Arc<Client>>>,
}

impl Broker {
    /// Creates a new broker instance by stating its configuration.
    /// The instance is not started yet and must be given to `start`.
    pub fn new(settings: BrokerSettings) -> Self {
        Broker {
            settings: settings.into(),
            clients: Default::default(),
        }
    }
}

impl Default for Broker {
    /// Creates a new broker instance with default configuration.
    /// Equivalent to calling `Broker::new(BrokerSettings::default())`
    fn default() -> Self {
        Broker::new(Default::default())
    }
}

impl From<BrokerSettings> for Broker {
    /// Creates a new broker instance from a configuration structure.
    /// Equivalent to calling `Broker::new(settings)`
    fn from(settings: BrokerSettings) -> Self {
        Broker::new(settings)
    }
}
