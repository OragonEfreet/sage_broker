use crate::service::Peer;
use async_std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};
use sage_mqtt::Packet;

pub enum Event {
    NewPeer(TcpStream),
    Control(Arc<Mutex<Peer>>, Packet),
    EndPeer(Arc<Mutex<Peer>>),
}
