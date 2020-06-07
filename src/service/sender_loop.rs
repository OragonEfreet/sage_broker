use crate::service::PacketReceiver;
use async_std::{net::TcpStream, prelude::*, sync::Arc};
use log::error;

pub async fn sender_loop(mut packets_receiver: PacketReceiver, stream: Arc<TcpStream>) {
    let mut stream = &*stream;
    while let Some(packet) = packets_receiver.next().await {
        let mut buffer = Vec::new();
        if let Err(e) = packet.encode(&mut buffer).await {
            error!("Cannot encode packet: {:?}", e);
        }
        if let Err(e) = stream.write_all(&buffer).await {
            error!("Cannot send packet: {:?}", e);
        }
    }
}
