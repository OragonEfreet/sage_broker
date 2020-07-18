use crate::{Broker, EventSender};
use async_std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct BrokerConfig {
    pub timeout_delay: u16,
    pub addr: String,
}

impl BrokerConfig {
    pub fn new(addr: &str) -> Self {
        BrokerConfig {
            timeout_delay: 10,
            addr: addr.into(),
        }
    }

    /// Sets the connection timeout delay in seconds, which is the amount of
    /// time the service will wait for a first connect packet before closing the
    /// connexion.
    pub fn with_connect_timeout_delay(mut self, delay: u16) -> Self {
        self.timeout_delay = delay;
        self
    }

    /// Builds a broker from the current configuration
    pub fn build(&self, event_sender: EventSender) -> Broker {
        Broker {
            config: RwLock::new(self.clone()),
            event_sender: RwLock::new(event_sender.clone()),
        }
    }
}
