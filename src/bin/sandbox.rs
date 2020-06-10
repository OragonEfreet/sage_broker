use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::TcpStream,
    task::{self, JoinHandle},
};
use sage_broker::{service, Broker};
use sage_mqtt::{Packet, ReasonCode};
use std::time::{Duration, Instant};

const TIMEOUT_DELAY: u16 = 3;

async fn prepare_connection() -> (JoinHandle<()>, TcpStream) {
    let handle = {
        let config = Broker::new("localhost:6788").with_connect_timeout_delay(TIMEOUT_DELAY);
        service::start(config)
    };

    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // pannic
    for _ in 0u8..5u8 {
        if let Ok(stream) = TcpStream::connect("localhost:6788").await {
            return (handle, stream);
        }

        task::sleep(Duration::from_secs(1)).await;
    }

    panic!("Cannot establish connection");
}

/// Requirements:
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
fn connect_timeout() {
    let (server, mut stream) = task::block_on(prepare_connection());

    task::block_on(async {
        let now = Instant::now();
        let delay_with_tolerance = (TIMEOUT_DELAY as f32 * 1.5) as u64;

        // Listen loop and sending nothing
        // loop {
        if let Ok(packet) = Packet::decode(&mut stream).await {
            match packet {
                Packet::ConnAck(packet) => {
                    assert_eq!(packet.reason_code, ReasonCode::UnspecifiedError)
                }
                _ => panic!("Server must send ConnAck packet only"),
            }
        }

        assert!(now.elapsed().as_secs() <= delay_with_tolerance);
        // }

        assert!(now.elapsed().as_secs() >= TIMEOUT_DELAY as u64);
    });

    task::block_on(server.cancel());
}

/// Requirements:
/// The Server MUST validate that the CONNECT packet matches the format
/// described in section 3.1 and close the Network Connection if it does not
/// match [MQTT-3.1.4-1].
/// The Server MAY send a CONNACK with a Reason Code of 0x80 or greater as
/// described in section 4.13 before closing the Network Connection.
fn mqtt_3_1_4_1() {
    let (server, mut stream) = task::block_on(prepare_connection());

    task::block_on(async {
        // Send an invalid connect packet and wait for an immediate disconnection
        // from the server.
        let packet = Packet::Connect(Default::default());
        let mut buffer = Vec::new();
        packet.encode(&mut buffer).await.unwrap();
        println!("Will send {:?}", buffer);

        while let Err(_) = stream.write(&buffer[..3]).await {println!(".");}

        // Wait for a response from the server within the next seconds
        let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
        let mut buf = vec![0u8; 1024];
        let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

        match result {
            Err(e) => match e.kind() {
                ErrorKind::TimedOut => panic!("Server did not respond to invalid connect packet"),
                _ => panic!("IO Error: {:?}", e),
            },
            Ok(0) => { /* Normal Disconnection */ }
            Ok(_) => {
                let mut buf = Cursor::new(buf);
                let packet = Packet::decode(&mut buf).await.unwrap();
                assert!(matches!(packet, Packet::ConnAck(_)));
                if let Packet::ConnAck(packet) = packet {
                    println!("==> {:?}", packet.reason_code);
                }
            }
        }
    });

    task::block_on(server.cancel());
}

fn main() {
    mqtt_3_1_4_1();
}