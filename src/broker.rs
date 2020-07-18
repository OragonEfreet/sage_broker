use crate::{BrokerConfig, Client, Event, EventSender};
use async_std::sync::RwLock;
use futures::SinkExt;
use log::error;

pub struct Broker {
    pub config: RwLock<BrokerConfig>,
    pub event_sender: RwLock<EventSender>,
    pub clients: Vec<Client>,
}

impl Broker {
    pub fn new(config: &BrokerConfig, event_sender: EventSender) -> Self {
        Broker {
            config: RwLock::new(config.clone()),
            event_sender: RwLock::new(event_sender.clone()),
            clients: Default::default(),
        }
    }

    pub async fn send(&self, event: Event) {
        if let Err(e) = self.event_sender.write().await.send(event).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
