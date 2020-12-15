use crate::{PacketSender, Session};
use async_std::sync::Arc;
use futures::SinkExt;
use log::error;
use sage_mqtt::Packet;

#[derive(Debug)]
pub struct Peer {
    session: Option<Arc<Session>>,
    packet_sender: PacketSender,
    closing: bool,
}

impl Peer {
    pub fn from_session(session: Arc<Session>, packet_sender: PacketSender) -> Self {
        Peer {
            session: Some(session),
            packet_sender,
            closing: false,
        }
    }

    pub fn new(packet_sender: PacketSender) -> Self {
        Peer {
            session: None,
            packet_sender,
            closing: false,
        }
    }

    pub fn session(&self) -> &Option<Arc<Session>> {
        &self.session
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
