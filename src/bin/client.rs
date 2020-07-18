use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::TcpStream,
    task::{self},
};
use sage_mqtt::Packet;
use std::time::Duration;

const TIMEOUT_DELAY: u16 = 3;

fn main() {
    task::block_on(async {
        let mut stream = TcpStream::connect("localhost:6788").await.unwrap();
        // Send an invalid connect packet and wait for an immediate disconnection
        // from the server.
        let packet = Packet::Connect(Default::default());
        let mut buffer = Vec::new();
        packet.encode(&mut buffer).await.unwrap();

        while let Err(_) = stream.write(&buffer).await {}

        // Wait for a response from the server within the next seconds
        let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
        let mut buf = vec![0u8; 1024];
        let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

        task::sleep(std::time::Duration::from_secs(3)).await;

        match result {
            Err(e) => match e.kind() {
                ErrorKind::TimedOut => panic!("Server did not respond to invalid connect packet"),
                _ => panic!("IO Error: {:?}", e),
            },
            Ok(0) => { /* Normal Disconnection */ }
            Ok(_) => {
                let mut buf = Cursor::new(buf);
                let packet = Packet::decode(&mut buf).await.unwrap();
                println!("Received: {:?}", packet);
            }
        }
        task::sleep(std::time::Duration::from_secs(3)).await;
    });
}
