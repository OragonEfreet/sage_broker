use crate::Broker;

pub struct BrokerBuilder {
    connect_timeout_delay: u16,
}

impl Default for BrokerBuilder {
    fn default() -> Self {
        BrokerBuilder {
            connect_timeout_delay: 10,
        }
    }
}

impl BrokerBuilder {
    pub fn start(&self) -> Broker {
        Broker {
            connect_timeout_delay: self.connect_timeout_delay,
        }
    }
}
