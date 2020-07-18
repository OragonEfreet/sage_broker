use crate::{Client, PacketSender, PeerState};
use async_std::{sync::Arc, task::JoinHandle};
use futures::SinkExt;
use log::error;
use sage_mqtt::Packet;

#[derive(Debug)]
pub struct Peer {
    client: Option<Arc<Client>>,
    packet_sender: PacketSender,
    sender_handle: JoinHandle<()>,
    state: PeerState,
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
            state: PeerState::New,
        }
    }

    pub fn new(packet_sender: PacketSender, sender_handle: JoinHandle<()>) -> Self {
        Peer {
            client: None,
            packet_sender,
            sender_handle,
            state: PeerState::New,
        }
    }

    pub fn client(&self) -> &Option<Arc<Client>> {
        &self.client
    }

    pub fn set_state(&mut self, state: PeerState) {
        self.state = state;
    }

    pub fn closing(&self) -> bool {
        self.state == PeerState::Closed
    }

    pub async fn close(&mut self, packet: Option<Packet>) {
        if let Some(packet) = packet {
            self.send(packet).await;
        }
        self.state = PeerState::Closed;
    }

    pub async fn send(&mut self, packet: Packet) {
        if let Err(e) = self.packet_sender.send(packet).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
