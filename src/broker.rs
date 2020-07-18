use crate::{BrokerConfig, Client, Event, EventSender};
use async_std::sync::{Arc, RwLock};
use futures::SinkExt;
use log::error;
use sage_mqtt::Connect;

pub struct Broker {
    pub config: RwLock<BrokerConfig>,
    pub event_sender: RwLock<EventSender>,
    pub clients: Vec<Arc<Client>>,
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

    /// This function analyses the incoming connect request and process with an
    /// answer.
    /// The connection involves a packet to send back to the client and 
    /// The creation / change of a Client instance.
    /// An existing client does not meant an MQTT connection is established.
    /// Connect/Ack handshake may be involved before the actuall session begins.
    pub fn connect(&mut self, connect: Connect) -> Option<Arc<Client>> {

        // Just create a new client for now
        let client = Arc::new(Client::new());
        self.clients.push(client.clone());

        Some(client)
    }
}
