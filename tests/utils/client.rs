use crate::utils::TIMEOUT_DELAY;
use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::{SocketAddr, TcpStream},
    task,
};
use sage_mqtt::Packet;
use std::time::Duration;

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
/// Sends the given packet and wait for the next response from the server.
/// Note that nothing ensures the received packet from the server is a response
/// to the sent packet.
pub async fn send_waitback(
    stream: &mut TcpStream,
    packet: Packet,
    invalid: bool,
) -> Option<Packet> {
    // Send the packet as buffer
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();

    if invalid {
        buffer[0] |= 0b1111; // Invalidate the packet
    }
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
/// Wait for the given client stream to be closed by the sever.
/// This version does not care about how the server closes the stream and
/// just wait for an Ok(0) from stream.read().
/// More fine grained function could test for no error or DISCONNECT packet
pub async fn wait_close(stream: &mut TcpStream) -> bool {
    loop {
        let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
        let mut buf = vec![0u8; 1024];
        match io::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
            Err(e) => match e.kind() {
                ErrorKind::TimedOut => return false,
                _ => continue,
            },
            Ok(0) => return true,
            _ => continue,
        }
    }
}
