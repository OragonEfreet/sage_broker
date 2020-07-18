use crate::{Broker, Event, Peer};
use async_std::{
    future,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, RwLock},
    task,
};
use log::{error, info};
use sage_mqtt::{ConnAck, Packet, ReasonCode};
use std::time::Duration;

pub fn listen_peer(peer: Arc<RwLock<Peer>>, broker: Arc<Broker>, stream: Arc<TcpStream>) {
    task::spawn(async move {
        info!("Start listening peer ({})", task::current().id());
        let timeout_delay = broker.config.read().await.timeout_delay;
        let out_time = Duration::from_secs(((timeout_delay as f32) * 1.5) as u64);
        let mut stream = BufReader::new(&*stream);
        loop {
            if let Ok(packet) = future::timeout(out_time, Packet::decode(&mut stream)).await {
                if peer.read().await.closing() {
                    // We just drop
                    break;
                }

                match packet {
                    Ok(packet) => broker.send(Event::Control(peer.clone(), packet)).await,
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
                if !peer.read().await.closing() {
                    let packet = ConnAck {
                        reason_code: ReasonCode::UnspecifiedError,
                        ..Default::default()
                    };
                    peer.write().await.send(packet.into()).await;
                }
                break;
            }
        }

        broker.send(Event::EndPeer(peer.clone())).await;
        info!("Stop listening peer ({})", task::current().id());
    });
}
