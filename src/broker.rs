use crate::BrokerService;

pub struct Broker {
    timeout_delay: u16,
}

impl Default for Broker {
    fn default() -> Self {
        Broker { timeout_delay: 10 }
    }
}

impl Broker {
    pub fn start(&self) -> BrokerService {
        BrokerService {
            timeout_delay: self.timeout_delay,
        }
    }

    /// Sets the connection timeout delay in seconds, which is the amount of
    /// time the service will wait for a first connect packet before closing the
    /// connexion.
    pub fn with_connect_timeout_delay(mut self, delay: u16) -> Self {
        self.timeout_delay = delay;
        self
    }
}
