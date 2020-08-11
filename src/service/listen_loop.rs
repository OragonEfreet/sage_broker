use crate::{Event, EventSender, Peer};
use async_std::{
    future,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, RwLock},
    task,
};
use futures::SinkExt;
use log::{error, info};
use sage_mqtt::{ConnAck, Disconnect, Packet, ReasonCode};
use std::time::Duration;

pub async fn listen_loop(
    peer: Arc<RwLock<Peer>>,
    mut event_sender: EventSender,
    timeout_delay: u16,
    stream: Arc<TcpStream>,
) {
    info!("Start listening peer ({})", task::current().id());
    let out_time = Duration::from_secs(((timeout_delay as f32) * 1.5) as u64);
    info!("Time out is {:?}", out_time);
    let mut stream = BufReader::new(&*stream);
    loop {
        // future::timeout returns a Result<T, TimeoutError>
        // T is a Result<Packet, Error>
        if let Ok(packet) = future::timeout(out_time, Packet::decode(&mut stream)).await {
            // If the connexion has been closed by some other task, we just
            // quit from here.
            if peer.read().await.closing() {
                break;
            }

            match packet {
                // If the result is a packet, we create a packet event
                Ok(packet) => {
                    if let Err(e) = event_sender
                        .send(Event::Control(peer.clone(), packet))
                        .await
                    {
                        error!("Cannot send event: {:?}", e);
                    }
                }
                // If it's an error (usually ProtocolError o MalformedPacket),
                // We ConnAck it and end the connection.
                Err(e) => {
                    error!("Error: {:?}", e);
                    let packet = ConnAck {
                        reason_code: e.into(),
                        ..Default::default()
                    };
                    peer.write().await.send(packet.into()).await;
                    break;
                }
            }
        } else {
            // If the peer is not in a closing state we can send a Disconnect
            // packet with KeepAliveTimeout reason code
            if !peer.read().await.closing() {
                let packet = Disconnect {
                    reason_code: ReasonCode::KeepAliveTimeout,
                    ..Default::default()
                };
                peer.write().await.send(packet.into()).await;
            }
            break;
        }
    }

    if let Err(e) = event_sender.send(Event::EndPeer(peer.clone())).await {
        error!("Cannot send event: {:?}", e);
    }

    info!("Stop listening peer ({})", task::current().id());
}
