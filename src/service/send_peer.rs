use crate::PacketReceiver;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

/// This function loop-reads from the given `PacketReceiver` for any incoming
/// `Packet`. Each of them is then encoded and sent to `stream`.
/// Once all senders are dropped, the receiver is dropped as well and the loop
/// is broken, ending the function.
/// The sender is held in a `Peer` instance.
pub async fn send_peer(mut from_packet_channel: PacketReceiver, mut stream: OwnedWriteHalf) {
    log::info!("Start send loop for '{}'", stream.peer_addr().unwrap());
    while let Some(packet) = from_packet_channel.recv().await {
        log::info!(">>> {:#?}", packet);
        let mut buffer = Vec::new();
        if let Err(e) = packet.encode(&mut buffer).await {
            log::error!("Cannot encode packet: {:#?}", e);
        }
        if let Err(e) = stream.write_all(&buffer).await {
            log::error!("Cannot send packet: {:#?}", e);
        }
    }
    log::info!("Stop send loop for '{}'", stream.peer_addr().unwrap());
}
