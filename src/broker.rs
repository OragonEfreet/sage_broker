use crate::BrokerConfig;
use async_std::sync::Arc;

pub struct Broker {
    pub config: Arc<BrokerConfig>,
}

impl Broker {
    pub fn from_config(config: BrokerConfig) -> Self {
        Broker {
            config: Arc::new(config),
        }
    }
}
