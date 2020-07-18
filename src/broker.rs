use crate::{BrokerConfig, Event, EventSender};
use async_std::sync::RwLock;
use futures::SinkExt;
use log::error;

pub struct Broker {
    pub config: RwLock<BrokerConfig>,
    pub event_sender: EventSender,
}

impl Broker {
    pub async fn send(&self, event: Event) {
        if let Err(e) = self.event_sender.clone().send(event).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
