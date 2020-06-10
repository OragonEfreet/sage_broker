use crate::service::PacketSender;
use async_std::task::JoinHandle;
use futures::SinkExt;
use log::error;
use nanoid::nanoid;
use sage_mqtt::Packet;

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

    pub async fn send(&mut self, packet: Packet) {
        if let Err(e) = self.packet_sender.send(packet).await {
            error!("Cannot send packet to channel: {:?}", e);
        }
    }
}
