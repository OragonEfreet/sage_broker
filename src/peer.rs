use crate::{PacketSender, Session, Trigger};
use log::error;
use sage_mqtt::Packet;
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock, Weak},
};

#[derive(Debug)]
pub struct Peer {
    addr: SocketAddr,
    session: RwLock<Weak<Session>>,
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

    pub fn bind(&self, new_session: Arc<Session>) {
        if let Ok(mut session) = self.session.write() {
            *session = Arc::downgrade(&new_session);
        } else {
            log::error!("Cannot write-lock session");
        }
    }

    pub fn session(&self) -> Option<Arc<Session>> {
        if let Ok(session) = self.session.read() {
            session.upgrade()
        } else {
            log::error!("Cannot read-lock session");
            None
        }
    }

    pub fn closing(&self) -> bool {
        self.closing.is_fired()
    }

    pub fn close(&self) {
        self.closing.fire();
    }

    pub fn send_close(&self, packet: Packet) {
        self.send(packet);
        self.close();
    }

    pub fn send(&self, packet: Packet) {
        if let Err(e) = self.packet_sender.send(packet) {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
