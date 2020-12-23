use crate::utils::TIMEOUT_DELAY;
use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::TcpStream,
};
use sage_mqtt::Packet;
use std::time::Duration;

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
