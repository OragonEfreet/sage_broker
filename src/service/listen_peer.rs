use crate::{BrokerSettings, CommandSender, Peer, Trigger};
use async_std::{
    future,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, RwLock},
};
use futures::SinkExt;
use log::{debug, error, info};
use sage_mqtt::{Disconnect, Packet, ReasonCode};
use std::time::{Duration, Instant};

/// The listen peer task is responsible for listening any incoming packet from a specific peer
/// and convert it to a MQTT packet. Once converted, it sends it into the commands channel.
pub async fn listen_peer(
    peer: Peer,
    mut to_command_channel: CommandSender,
    settings: Arc<BrokerSettings>,
    stream: Arc<TcpStream>,
    shutdown: Trigger,
) {
    let peer = Arc::new(RwLock::new(peer));
    info!("Start listening from '{}'", peer.read().await.addr(),);
    // If the keep alive is 0, timeout_delay is set to 3 but the timeout error
    // won't disconnect the peer.
    // If the keep alive is > 0 timeout_delay is 1.5 times it, and a timeout will
    // disconnect
    let mut keep_alive = {
        let keep_alive = settings.keep_alive;

        if keep_alive == 0 {
            info!("Time out is disabled");
            None
        } else {
            let max = Duration::from_secs(((keep_alive as f32) * 1.5) as u64);
            info!("Time out is {:?} (1.5*{:?})", max, keep_alive);
            Some((max, Instant::now()))
        }
    };
    let timeout_delay = Duration::from_secs(1_u64);

    let mut stream = BufReader::new(&*stream);
    while !peer.read().await.closing() {
        if let Some((max, last)) = keep_alive {
            debug!("KeepAlive: {:?}/{:?}", last.elapsed(), max);
        }
        // If the server is closing, we close the peer too and break
        if shutdown.is_fired().await {
            let packet = Disconnect {
                reason_code: ReasonCode::ServerShuttingDown,
                ..Default::default()
            };
            peer.write().await.send_close(packet.into()).await;
            break;
        }

        // future::timeout returns a Result<T, TimeoutError>
        // T is a Result<Packet, Error>
        if let Ok(decoded) = future::timeout(timeout_delay, Packet::decode(&mut stream)).await {
            // At this point, decoded may be an `Err(Io(Kind(UnexpectedEof)))`
            // But it's only considered an error if the peer was not is close state.

            // If the connexion has been closed by some other task, we just
            // quit from here.
            if peer.read().await.closing() {
                break;
            }

            match decoded {
                // If the result is a packet, we create a packet command
                Ok(packet) => {
                    if let Err(e) = to_command_channel.send((peer.clone(), packet)).await {
                        error!("Cannot send command: {:?}", e);
                    }
                }
                // If it's an error (usually ProtocolError o MalformedPacket),
                // We ConnAck it and end the connection.
                Err(e) => {
                    error!("Decode Error: {:?}", e);

                    if peer.read().await.session().is_some() {
                        let packet = Disconnect {
                            reason_code: e.into(),
                            ..Default::default()
                        };
                        peer.write().await.send_close(packet.into()).await;
                    } else {
                        peer.write().await.close().await;
                    }
                }
            }

            // Reset the keep alive timer
            if let Some((max, _)) = keep_alive {
                keep_alive = Some((max, Instant::now()));
            };
        } else if let Some((max, last)) = keep_alive {
            if last.elapsed() > max {
                // If keep_alive is activated
                info!("Peer timout, send Disconnect");
                // If the peer is not in a closing state we can send a Disconnect
                // packet with KeepAliveTimeout reason code
                if !peer.read().await.closing() {
                    let packet = Disconnect {
                        reason_code: ReasonCode::KeepAliveTimeout,
                        ..Default::default()
                    };
                    peer.write().await.send_close(packet.into()).await;
                }
            }
        }
    }

    info!("Stop listening from '{}'", peer.read().await.addr(),);
}
