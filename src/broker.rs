use crate::{BrokerConfig, EventSender};
use async_std::sync::Arc;

pub struct Broker {
    pub config: Arc<BrokerConfig>,
    pub event_sender: EventSender,
}
