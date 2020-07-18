use crate::BrokerConfig;
use async_std::sync::Arc;

pub struct Broker {
    pub config: Arc<BrokerConfig>,
}
