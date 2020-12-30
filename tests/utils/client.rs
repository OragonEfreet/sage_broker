use crate::utils::TIMEOUT_DELAY;
use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::{SocketAddr, TcpStream},
    task,
};
use sage_mqtt::{Packet, ReasonCode};
use std::time::Duration;

///////////////////////////////////////////////////////////////////////////////
// Create a valid TcpStream client for the current server
pub async fn spawn(local_addr: &SocketAddr) -> Option<TcpStream> {
    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // panic
    for _ in 0u8..5u8 {
        if let Ok(stream) = TcpStream::connect(*local_addr).await {
            return Some(stream);
        }

        task::sleep(Duration::from_secs(1)).await;
    }

    None
}

///////////////////////////////////////////////////////////////////////////////
/// Sends the given data and wait for the next response from the server.
/// Note that nothing ensures the received packet from the server is a response
/// to the sent packet.
pub async fn send_waitback_data(stream: &mut TcpStream, buffer: Vec<u8>) -> Option<Packet> {
    while stream.write(&buffer).await.is_err() {}

    let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
    let mut buf = vec![0u8; 1024];
    match io::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
        Err(e) => match e.kind() {
            ErrorKind::TimedOut => panic!("Server did not respond to invalid connect packet"),
            _ => panic!("IO Error: {:?}", e),
        },
        Ok(0) => None,
        Ok(_) => {
            let mut buf = Cursor::new(buf);
            Some(Packet::decode(&mut buf).await.unwrap())
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Sends the given packet and wait for the next response from the server.
/// Note that nothing ensures the received packet from the server is a response
/// to the sent packet.
pub async fn send_waitback(stream: &mut TcpStream, packet: Packet) -> Option<Packet> {
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
    let delay_with_tolerance = Duration::from_secs(10);
    let mut buf = vec![0u8; 1024];

    match io::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
        Err(e) => match e.kind() {
            ErrorKind::TimedOut => Some("Server did not respond"),
            _ => Some("IO Error"),
        },
        Ok(0) => {
            // Connexion shutdown with no disconnect
            match disconnect {
                DisPacket::Ignore(_) | DisPacket::Forbid => None,
                DisPacket::Force(_) => Some("DISCONNECT packet expected before close"),
            }
        }
        Ok(_) => {
            // DISCONNECT packet sent before shut down
            if let DisPacket::Forbid = disconnect {
                Some("Server should have closed with no prior packet")
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
                            Some("Incorrect reason_code sent for DISCONNECT")
                        }
                        _ => {
                            let mut buf = vec![0u8; 1024];
                            // Here the server MUST close the connection.
                            // Thus we only expect a Ok(0)
                            match io::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
                                Err(e) => match e.kind() {
                                    ErrorKind::TimedOut => {
                                        Some("Server did not respond after DISCONNECT packet")
                                    }
                                    _ => Some("IO Error"),
                                },
                                Ok(0) => None,
                                Ok(_) => Some("Unexpected packet send instead of close"),
                            }
                        }
                    }
                } else {
                    Some("Only DISCONNECT packet is allowed before closing")
                }
            }
        }
    }
    .map(Into::into) // Converts Option<&str> into Option<String>
}
