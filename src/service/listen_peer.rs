use crate::service::{Event, EventSender, Peer};
use async_std::{
    future,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, Mutex},
    task,
};
use futures::SinkExt;
use log::{error, info};
use sage_mqtt::Packet;
use std::time::Duration;

pub fn listen_peer(
    peer: Peer,
    timeout_delay: u16,
    mut event_sender: EventSender,
    stream: Arc<TcpStream>,
) {
    let peer = Arc::new(Mutex::new(peer));

    let peer_addr = stream.peer_addr().unwrap();
    let out_time = Duration::from_secs(timeout_delay as u64);

    task::spawn(async move {
        let mut stream = BufReader::new(&*stream);
        loop {
            if let Ok(packet) = future::timeout(out_time, Packet::decode(&mut stream)).await {
                match packet {
                    Ok(packet) => {
                        if let Err(e) = event_sender.send(Event::Control(packet)).await {
                            error!("Cannot generate control event: {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        break;
                    }
                }
            } else {
                error!("Time out");
                break;
            }
        }

        if let Err(e) = event_sender.send(Event::EndPeer(peer.clone())).await {
            error!("Cannot send EndPeer event: {:?}", e);
        }
        info!("Close {}", peer_addr);
    });
}
