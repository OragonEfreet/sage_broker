use async_std::net::TcpStream;
use sage_mqtt::Packet;

pub enum Event {
    NewPeer(TcpStream),
    Control(Packet),
}
