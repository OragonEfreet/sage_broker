use crate::{Broker, ControlSender, Peer};
use async_std::{
    future,
    io::BufReader,
    net::TcpStream,
    sync::{Arc, RwLock},
};
use futures::SinkExt;
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Disconnect, Packet, ReasonCode};
use std::time::Duration;

pub async fn listen_peer(
    peer: Arc<RwLock<Peer>>,
    mut to_control_channel: ControlSender,
    broker: Arc<Broker>,
    stream: Arc<TcpStream>,
) {
    let addr = if let Ok(addr) = stream.peer_addr() {
        addr.to_string()
    } else {
        "N/A".into()
    };

    info!("Start listening from '{}'", addr,);
    // If the keep alive is 0, timeout_delay is set to 3 but the timeout error
    // won't disconnect the peer.
    // If the keep alive is > 0 timeout_delay is 1.5 times it, and a timeout will
    // disconnect
    let (disconnect_on_timeout, timeout_delay) = {
        let timeout_delay = broker.settings.read().await.keep_alive;

        if timeout_delay == 0 {
            info!("Time out is disabled");
            (false, Duration::from_secs(3_u64))
        } else {
            let timeout_delay = Duration::from_secs(((timeout_delay as f32) * 1.5) as u64);
            info!("Time out is {:?}", timeout_delay);
            (true, timeout_delay)
        }
    };

    let mut stream = BufReader::new(&*stream);
    while !peer.read().await.closing() {
        // If the server is closing, we close the peer too and break
        if broker.is_shutting_down().await {
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
                // If the result is a packet, we create a packet control
                Ok(packet) => {
                    if let Err(e) = to_control_channel.send((peer.clone(), packet).into()).await {
                        error!("Cannot send control: {:?}", e);
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
                    peer.write().await.send_close(packet.into()).await;
                }
            }
        } else {
            if disconnect_on_timeout {
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

    info!("Stop listening from '{}'", addr);
}
