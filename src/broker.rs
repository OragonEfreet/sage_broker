use crate::BrokerSettings;
use async_std::sync::{Arc, RwLock};

/// The broker instance. Holds and the data related to broker configuration and
/// session management.
pub struct Broker {
    /// The broker configuration
    pub settings: RwLock<BrokerSettings>,
    shutdown: RwLock<bool>,
}

impl Broker {
    /// Creates a new Arc<Broker> instance by stating its configuration.
    /// The instance is not started yet and must be given to `start`.
    pub fn build(settings: BrokerSettings) -> Arc<Self> {
        Broker {
            settings: settings.into(),
            shutdown: false.into(),
        }
        .into()
    }

    /// Asks for safe shutdown of the broker.
    /// The broker won't be stopped immediately, as running tasks will end
    /// as soon as they can do it safely.
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
    }

    /// Checks whether the broker is in shutdown state.
    /// If this value is `true`, it cannot be `false` again.
    pub async fn is_shutting_down(&self) -> bool {
        *self.shutdown.read().await
    }
}
