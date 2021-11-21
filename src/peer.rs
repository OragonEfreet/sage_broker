use crate::{PacketSender, Session, Trigger};
use async_std::{
    net::SocketAddr,
    sync::{Arc, RwLock, Weak},
};
use log::error;
use sage_mqtt::Packet;

#[derive(Debug)]
pub struct Peer {
    addr: SocketAddr,
    session: RwLock<Weak<RwLock<Session>>>,
    packet_sender: PacketSender,
    closing: Trigger,
}

impl Peer {
    pub fn new(addr: SocketAddr, packet_sender: PacketSender) -> Self {
        Peer {
            addr,
            packet_sender,
            session: Default::default(),
            closing: Default::default(),
        }
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub async fn bind(&mut self, session: Arc<RwLock<Session>>) {
        *(self.session.write().await) = Arc::downgrade(&session);
    }

    pub async fn session(&self) -> Option<Arc<RwLock<Session>>> {
        self.session.read().await.upgrade()
    }

    pub async fn closing(&self) -> bool {
        self.closing.is_fired().await
    }

    pub async fn close(&self) {
        self.closing.fire().await;
    }

    pub async fn send_close(&self, packet: Packet) {
        self.send(packet).await;
        self.close().await;
    }

    pub async fn send(&self, packet: Packet) {
        if let Err(e) = self.packet_sender.send(packet).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
