use crate::PacketReceiver;
use async_std::{net::TcpStream, prelude::*, sync::Arc};
use log::error;

/// This function loop-reads from the given `PacketReceiver` for any incoming
/// `Packet`. Each of them is then encoded and sent to `stream`.
/// Once all senders are dropped, the receiver is dropped as well and the loop
/// is broken, ending the function.
/// The sender is held in a `Peer` instance.
pub async fn send_loop(mut packets_receiver: PacketReceiver, stream: Arc<TcpStream>) {
    let mut stream = &*stream;
    while let Some(packet) = packets_receiver.next().await {
        log::debug!("Sending a {}", packet);
        let mut buffer = Vec::new();
        if let Err(e) = packet.encode(&mut buffer).await {
            error!("Cannot encode packet: {:?}", e);
        }
        if let Err(e) = stream.write_all(&buffer).await {
            error!("Cannot send packet: {:?}", e);
        }
    }
}
