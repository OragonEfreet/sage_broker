use crate::{Client, PacketSender};
use async_std::sync::Arc;
use futures::SinkExt;
use log::error;
use sage_mqtt::Packet;

#[derive(Debug)]
pub struct Peer {
    client: Option<Arc<Client>>,
    packet_sender: PacketSender,
    closing: bool,
}

impl Peer {
    pub fn from_client(client: Arc<Client>, packet_sender: PacketSender) -> Self {
        Peer {
            client: Some(client),
            packet_sender,
            closing: false,
        }
    }

    pub fn new(packet_sender: PacketSender) -> Self {
        Peer {
            client: None,
            packet_sender,
            closing: false,
        }
    }

    pub fn client(&self) -> &Option<Arc<Client>> {
        &self.client
    }

    pub fn closing(&self) -> bool {
        self.closing
    }

    pub async fn close(&mut self) {
        self.closing = true;
    }

    pub async fn send_close(&mut self, packet: Packet) {
        self.send(packet).await;
        self.close().await;
    }

    pub async fn send(&mut self, packet: Packet) {
        if let Err(e) = self.packet_sender.send(packet).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
