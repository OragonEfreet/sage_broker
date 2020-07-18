use crate::Peer;
use async_std::{
    net::TcpStream,
    sync::{Arc, RwLock},
};
use sage_mqtt::Packet;

pub enum Event {
    NewPeer(TcpStream),
    Control(Arc<RwLock<Peer>>, Packet),
    EndPeer(Arc<RwLock<Peer>>),
}
