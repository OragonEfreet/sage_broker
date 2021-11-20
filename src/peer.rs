use crate::{PacketSender, Session};
use async_std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use log::error;
use sage_mqtt::Packet;

#[derive(Debug)]
pub struct Peer {
    addr: SocketAddr,
    session: Option<Arc<RwLock<Session>>>,
    packet_sender: PacketSender,
    closing: bool,
}

impl Peer {
    pub fn new(addr: SocketAddr, packet_sender: PacketSender) -> Self {
        Peer {
            addr,
            session: None,
            packet_sender,
            closing: false,
        }
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn bind(&mut self, session: Arc<RwLock<Session>>) {
        self.session = Some(session);
    }

    pub fn session(&self) -> &Option<Arc<RwLock<Session>>> {
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
