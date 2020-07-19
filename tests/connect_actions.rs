use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::TcpStream,
    task::{self, JoinHandle},
};
use sage_broker::Broker;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::{Duration, Instant};

mod setup;

const TIMEOUT_DELAY: u16 = 3;

fn prepare_connection() -> (JoinHandle<()>, TcpStream) {
    let config = Broker::new("localhost:6788").with_connect_timeout_delay(TIMEOUT_DELAY);
    setup::prepare_connection(config)
}

/// Requirements:
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
#[test]
fn connect_timeout() {
    let (server, mut stream) = prepare_connection();

    task::block_on(async {
        let now = Instant::now();
        let delay_with_tolerance = (TIMEOUT_DELAY as f32 * 1.5) as u64;

        if let Ok(packet) = Packet::decode(&mut stream).await {
            match packet {
                Packet::ConnAck(packet) => {
                    assert_eq!(packet.reason_code, ReasonCode::UnspecifiedError)
                }
                _ => panic!("Server must send ConnAck packet only"),
            }
        }

        assert!(now.elapsed().as_secs() >= delay_with_tolerance);
    });

    task::block_on(server.cancel());
}

/// Requirements:
/// The Server MUST validate that the CONNECT packet matches the format
/// described in section 3.1 and close the Network Connection if it does not
/// match [MQTT-3.1.4-1].
/// The Server MAY send a CONNACK with a Reason Code of 0x80 or greater as
/// described in section 4.13 before closing the Network Connection.
#[test]
fn mqtt_3_1_4_1() {
    let (server, mut stream) = prepare_connection();

    task::block_on(async {
        // Send an invalid connect packet and wait for an immediate disconnection
        // from the server.
        let packet = Packet::Connect(Default::default());
        let mut buffer = Vec::new();
        packet.encode(&mut buffer).await.unwrap();
        buffer[0] |= 0b1111; // Invalidate the packet

        while let Err(_) = stream.write(&buffer).await {}

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
                    assert_eq!(packet.reason_code, ReasonCode::MalformedPacket);
                }
            }
        }
    });

    task::block_on(server.cancel());
}

/// The Server MAY check that the contents of the CONNECT packet meet any
/// further restrictions and SHOULD perform authentication and authorization
/// checks. If any of these checks fail, it MUST close the Network Connection
/// [MQTT-3.1.4-2].
/// Before closing the Network Connection, it MAY send an appropriate CONNACK
/// response with a Reason Code of 0x80 or greater as described in section 3.2
/// and section 4.13.
#[test]
fn mqtt_3_1_4_2() {
    // Create a set of pairs with a server and an unsupported connect packet.
    // Each one will be tested against an expected ConnAck > 0x80
    let config = Broker::new("localhost:6788");
    let test_pairs = vec![(
        config.clone(),
        Connect {
            authentication: Some(Default::default()),
            ..Default::default()
        },
    )];

    for (config, connect) in test_pairs {
        let (server, mut stream) = setup::prepare_connection(config);

        task::block_on(async {
            // Send an invalid connect packet and wait for an immediate disconnection
            // from the server.
            let packet = Packet::from(connect);
            let mut buffer = Vec::new();
            packet.encode(&mut buffer).await.unwrap();
            buffer[0] |= 0b1111; // Invalidate the packet

            while let Err(_) = stream.write(&buffer).await {}

            // Wait for a response from the server within the next seconds
            let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
            let mut buf = vec![0u8; 1024];
            let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

            match result {
                Err(e) => match e.kind() {
                    ErrorKind::TimedOut => {
                        panic!("Server did not respond to invalid connect packet")
                    }
                    _ => panic!("IO Error: {:?}", e),
                },
                Ok(0) => { /* Normal Disconnection */ }
                Ok(_) => {
                    let mut buf = Cursor::new(buf);
                    let packet = Packet::decode(&mut buf).await.unwrap();
                    if let Packet::ConnAck(packet) = packet {
                        assert!(packet.reason_code as u16 >= 0x80);
                    } else {
                        panic!("Packet should be ConnAck");
                    }
                }
            }
        });

        task::block_on(server.cancel());
    }
}
