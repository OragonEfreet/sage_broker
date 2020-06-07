use crate::service::PacketSender;
use async_std::task::JoinHandle;
use nanoid::nanoid;

#[derive(Debug)]
pub struct Peer {
    client_id: String,
    packet_sender: PacketSender,
    sender_handle: JoinHandle<()>,
}

impl Peer {
    pub fn new(packet_sender: PacketSender, sender_handle: JoinHandle<()>) -> Self {
        let client_id = format!("sage_mqtt-{}", nanoid!());
        Peer {
            client_id,
            packet_sender,
            sender_handle,
        }
    }
}
