use crate::{Broker, Client, Event, EventSender, Peer, ClientStatus};
use async_std::sync::{Arc, RwLock};
use futures::SinkExt;
use log::error;
use sage_mqtt::{ConnAck, Connect};

pub struct ToDelete {
    pub config: RwLock<Broker>,
    pub event_sender: RwLock<EventSender>,
    pub clients: RwLock<Vec<Arc<Client>>>,
}

impl ToDelete {
    pub fn new(config: &Broker, event_sender: EventSender) -> Self {
        ToDelete {
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
    pub async fn connect(&self, peer: Arc<RwLock<Peer>>, _: Connect) {
        // Just create a new client for now
        let client = Arc::new(Client::new(peer.clone()));
        self.clients.write().await.push(client.clone());
        // TODO information may be taken from the packet to customize the client

        // Send an ok ConnAck packet to the client
        // client.status = ClientStatus::Ready;
        peer.write().await.send(ConnAck::default().into()).await;
    }
}
