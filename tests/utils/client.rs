use crate::utils::TIMEOUT_DELAY;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::{io::Cursor, net::SocketAddr, time::Duration};
use tokio::{io::AsyncReadExt, io::AsyncWriteExt, net::TcpStream, time};

///////////////////////////////////////////////////////////////////////////////
// Create a valid TcpStream client for the current server
pub async fn spawn(local_addr: &SocketAddr) -> TcpStream {
    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // panic
    for _ in 0u8..5u8 {
        if let Ok(stream) = TcpStream::connect(*local_addr).await {
            return stream;
        }

        time::sleep(Duration::from_secs(1)).await;
    }

    panic!("Cannot spawn client after 5 attempts");
}

///////////////////////////////////////////////////////////////////////////////
/// Create a valid TcpStream client and send a connect request, returning the
/// stream in case of success
pub async fn connect(local_addr: &SocketAddr, connect: Connect) -> (TcpStream, Option<String>) {
    let mut stream = spawn(&local_addr).await;

    if let Response::Packet(Packet::ConnAck(connack)) =
        send_waitback(&mut stream, Packet::Connect(connect)).await
    {
        assert_eq!(connack.reason_code, ReasonCode::Success);
        (stream, connack.assigned_client_id.clone())
    } else {
        panic!("Expected CONNACK(Success) packet");
    }
}

/// The kind of response send by send_waitback_data and send_waitback
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Response {
    /// The server did not send anything after a given delay
    None,
    /// The server closed the connection
    Close,
    /// The server returned a packet
    Packet(Packet),
}
///////////////////////////////////////////////////////////////////////////////
/// Sends the given data and wait for the next response from the server.
/// Note that nothing ensures the received packet from the server is a response
/// to the sent packet.
pub async fn send_waitback_data(stream: &mut TcpStream, buffer: Vec<u8>) -> Response {
    while stream.write(&buffer).await.is_err() {}

    let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
    let mut buf = vec![0u8; 1024];

    // Result<Result<usize, std::io::Error>, Elapsed>

    if let Ok(response) = time::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
        match response {
            Err(e) => panic!("IO Error: {:?}", e),
            Ok(0) => Response::Close,
            Ok(_) => Response::Packet(Packet::decode(&mut Cursor::new(buf)).await.unwrap()),
        }
    } else {
        Response::None
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Sends the given packet and wait for the next response from the server.
/// Note that nothing ensures the received packet from the server is a response
/// to the sent packet.
pub async fn send_waitback(stream: &mut TcpStream, packet: Packet) -> Response {
    // Send the packet as buffer
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    send_waitback_data(stream, buffer).await
}

///////////////////////////////////////////////////////////////////////////////
/// Used within `wait_close` to specify how the Disconnect packet behaviour is
/// expected.
pub enum DisPacket {
    /// No specific requirement
    /// The function will succeed wether a Disconnect packet is sent before close or not
    /// If a reason code is set, it MUST match
    Ignore(Option<ReasonCode>),
    /// A Disconnect packet MUST be sent. If a ReasonCode is set, it MUST match
    Force(Option<ReasonCode>),
    /// A Disconnect packet must NOT be sent
    Forbid,
}

///////////////////////////////////////////////////////////////////////////////
/// Wait for the given client stream to be closed by the sever.
/// The function can expect/forbid/ignore a disconnect packet prior to closing
/// using the `disconnect` option.
/// In case of success, the function returns None. Any error is returns as
/// an Some(String)
pub async fn wait_close(mut stream: TcpStream, disconnect: DisPacket) -> Option<String> {
    let delay_with_tolerance = Duration::from_secs(TIMEOUT_DELAY as u64);
    let mut buf = vec![0u8; 1024];

    if let Ok(response) = time::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
        match response {
            Err(_) => Some("IO Error".to_string()),
            Ok(0) => {
                // Connexion shutdown with no disconnect
                match disconnect {
                    DisPacket::Ignore(_) | DisPacket::Forbid => None,
                    DisPacket::Force(_) => {
                        Some("DISCONNECT packet expected before close".to_string())
                    }
                }
            }
            Ok(_) => {
                // DISCONNECT packet sent before shut down
                if let DisPacket::Forbid = disconnect {
                    Some("Server should have closed with no prior packet".to_string())
                } else {
                    // disconnect can only be Ignore of Force(_) here.
                    // In both cases, the optionally received packet MUST be a DISCONNECT
                    let mut buf = Cursor::new(buf);
                    let packet = Packet::decode(&mut buf).await.unwrap();
                    if let Packet::Disconnect(packet) = packet {
                        match disconnect {
                            DisPacket::Force(Some(rc)) | DisPacket::Ignore(Some(rc))
                                if rc != packet.reason_code =>
                            {
                                Some(format!(
                                    "Expected reason code '{:?}', got '{:?}'",
                                    rc, packet.reason_code
                                ))
                            }
                            _ => {
                                let mut buf = vec![0u8; 1024];
                                // Here the server MUST close the connection.
                                // Thus we only expect a Ok(0)

                                if let Ok(result) =
                                    time::timeout(delay_with_tolerance, stream.read(&mut buf)).await
                                {
                                    match result {
                                        Err(_) => Some("IO Error".to_string()),
                                        Ok(0) => None,
                                        Ok(_) => Some(
                                            "Unexpected packet send instead of close".to_string(),
                                        ),
                                    }
                                } else {
                                    Some(
                                        "Server did not respond after DISCONNECT packet"
                                            .to_string(),
                                    )
                                }
                            }
                        }
                    } else {
                        Some("Only DISCONNECT packet is allowed before closing".to_string())
                    }
                }
            }
        }
    } else {
        Some("Server did not respond".to_string())
    }
}
