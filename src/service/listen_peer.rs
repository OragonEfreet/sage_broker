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
use sage_mqtt::{ConnAck, Packet, ReasonCode};
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
                info!("Receveid something");

                if peer.lock().await.closing() {
                    // We just drop
                    break;
                }

                match packet {
                    Ok(packet) => {
                        if let Err(e) = event_sender
                            .send(Event::Control(peer.clone(), packet))
                            .await
                        {
                            error!("Cannot generate control event: {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        let packet = ConnAck {
                            reason_code: e.into(),
                            ..Default::default()
                        };
                        peer.lock().await.send(packet.into()).await;
                        break;
                    }
                }
            } else {
                if !peer.lock().await.closing() {
                    log::debug!("Timeout");
                    let packet = ConnAck {
                        reason_code: ReasonCode::UnspecifiedError,
                        ..Default::default()
                    };
                    peer.lock().await.send(packet.into()).await;
                }
                break;
            }
        }

        if let Err(e) = event_sender.send(Event::EndPeer(peer.clone())).await {
            error!("Cannot send EndPeer event: {:?}", e);
        }
        info!("Close {}", peer_addr);
    });
}
