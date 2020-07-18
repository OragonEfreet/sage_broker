use crate::{Client, PacketSender};
use async_std::{sync::Arc, task::JoinHandle};
use futures::SinkExt;
use log::error;
use sage_mqtt::Packet;

#[derive(Debug)]
pub struct Peer {
    client: Option<Arc<Client>>,
    packet_sender: PacketSender,
    sender_handle: JoinHandle<()>,
    closing: bool,
}

impl Peer {
    pub fn from_client(
        client: Arc<Client>,
        packet_sender: PacketSender,
        sender_handle: JoinHandle<()>,
    ) -> Self {
        Peer {
            client: Some(client),
            packet_sender,
            sender_handle,
            closing: false,
        }
    }

    pub fn new(packet_sender: PacketSender, sender_handle: JoinHandle<()>) -> Self {
        Peer {
            client: None,
            packet_sender,
            sender_handle,
            closing: false,
        }
    }

    pub fn client(&self) -> &Option<Arc<Client>> {
        &self.client
    }

    pub fn closing(&self) -> bool {
        self.closing
    }

    pub async fn close(&mut self, packet: Option<Packet>) {
        if let Some(packet) = packet {
            self.send(packet).await;
        }
        self.closing = true;
    }

    pub async fn send(&mut self, packet: Packet) {
        if let Err(e) = self.packet_sender.send(packet).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
