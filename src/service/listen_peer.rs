use crate::service::{Event, EventSender};
use async_std::{future, io::BufReader, net::TcpStream, task};
use futures::SinkExt;
use log::{error, info};
use sage_mqtt::Packet;
use std::time::Duration;

pub fn listen_peer(timeout_delay: u16, mut event_sender: EventSender, stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream);

    // Create the peer info

    let out_time = Duration::from_secs(timeout_delay as u64);
    task::spawn(async move {
        loop {
            if let Ok(packet) = future::timeout(out_time, Packet::decode(&mut reader)).await {
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

        info!("Close {}", peer_addr);
    });
}
