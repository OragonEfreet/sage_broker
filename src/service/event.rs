use async_std::net::TcpStream;
use futures::channel::mpsc;

use sage_mqtt::Packet;

pub enum Event {
    NewPeer(TcpStream),
    Control(Packet),
}

pub type EventSender = mpsc::UnboundedSender<Event>;
pub type EventReceiver = mpsc::UnboundedReceiver<Event>;
