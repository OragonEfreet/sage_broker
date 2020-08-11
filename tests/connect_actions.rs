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
    let config = Broker {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    };
    setup::prepare_connection(config)
}

/// Requirements:
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
#[test]
fn connect_timeout() {
    pretty_env_logger::init();
    let (_, mut stream) = prepare_connection();

    task::block_on(async {
        let now = Instant::now();
        let delay_with_tolerance = (TIMEOUT_DELAY as f32 * 1.5) as u64;

        if let Ok(packet) = Packet::decode(&mut stream).await {
            match packet {
                Packet::Disconnect(packet) => {
                    assert_eq!(packet.reason_code, ReasonCode::KeepAliveTimeout)
                }
                _ => panic!("Server must send Disconnect packet only"),
            }
        }

        assert!(now.elapsed().as_secs() >= delay_with_tolerance);
    });

    // task::block_on(server.cancel());
}

/// Requirements:
/// The Server MUST validate that the CONNECT packet matches the format
/// described in section 3.1 and close the Network Connection if it does not
/// match [MQTT-3.1.4-1].
/// The Server MAY send a CONNACK with a Reason Code of 0x80 or greater as
/// described in section 4.13 before closing the Network Connection.
#[test]
fn mqtt_3_1_4_1() {
    let (_, mut stream) = prepare_connection();

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

    // task::block_on(server.cancel());
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
    let config = Broker {
        addr: "localhost:6788".into(),
        ..Default::default()
    };
    let test_data = vec![
        (
            config.clone(),
            Connect {
                authentication: Some(Default::default()),
                ..Default::default()
            },
            ReasonCode::BadAuthenticationMethod,
        ),
        (
            config.clone(),
            Connect {
                user_name: Some("Thanos".into()),
                ..Default::default()
            },
            ReasonCode::BadAuthenticationMethod,
        ),
        // (config.clone(), Connect::default()), <-- This one must fail
    ];

    for (config, connect, reason_code) in test_data {
        let (_, mut stream) = setup::prepare_connection(config);

        task::block_on(async {
            // Send an invalid connect packet and wait for an immediate disconnection
            // from the server.
            let packet = Packet::from(connect);
            let mut buffer = Vec::new();
            packet.encode(&mut buffer).await.unwrap();

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
                        assert_eq!(packet.reason_code, reason_code);
                    } else {
                        panic!("Packet should be ConnAck");
                    }
                }
            }
        });

        // task::block_on(server.cancel());
    }
}

/// If the ClientID represents a Client already connected to the Server, the
/// Server sends a DISCONNECT packet to the existing Client with Reason Code
/// of 0x8E (Session taken over) as described in section 4.13 and MUST close
/// the Network Connection of the existing Client [MQTT-3.1.4-3]
#[async_std::test]
async fn mqtt_3_1_4_3() {
    // Start server and first client
    let (_, mut stream) = {
        let config = Broker {
            keep_alive: 100,
            ..Default::default()
        };
        setup::prepare_connection(config)
    };

    ////////////////////////////////////////////////////////////////////////////
    // Using the first client, we send a Connect packet and wait for the ConnAck
    let packet = Packet::Connect(Connect {
        client_id: Some("Suzuki".into()),
        ..Default::default()
    });
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    while let Err(_) = stream.write(&buffer).await {}

    // Wait for a response from the server within the next seconds
    let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
    let mut buf = vec![0u8; 1024];
    let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

    match result {
        Err(e) => match e.kind() {
            ErrorKind::TimedOut => panic!("Server did not respond to connect packet"),
            _ => panic!("IO Error: {:?}", e),
        },
        Ok(0) => panic!("Server shut down connexion after Connect packet is sent"),
        Ok(_) => {
            let mut buf = Cursor::new(buf);
            let packet = Packet::decode(&mut buf).await.unwrap();
            if let Packet::ConnAck(packet) = packet {
                assert_eq!(packet.reason_code, ReasonCode::Success);
            } else {
                panic!("Invalid packet type sent after Connect");
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////
    // We spawn a new task which uses the first connection to wait for a
    // Disconnect(SessionTakenOver) packet without the next ten seconds.
    let wait_dis = task::spawn(async move {
        let delay_with_tolerance = Duration::from_secs(10);
        let mut buf = vec![0u8; 1024];
        let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

        match result {
            Err(e) => match e.kind() {
                ErrorKind::TimedOut => Some("Server did not respond"),
                _ => Some("IO Error"),
            },
            Ok(0) => Some("Server shut down connexion after Connect packet is sent"),
            Ok(_) => {
                let mut buf = Cursor::new(buf);
                let packet = Packet::decode(&mut buf).await.unwrap();
                if let Packet::Disconnect(_) = packet {
                    None
                } else {
                    Some("Incorrect packet type sent by server")
                }
            }
        }
    });

    ////////////////////////////////////////////////////////////////////////////
    // Meanwhile, we connect with a new connexion and the same client. This
    // must generate a Disconnect packet recevied by the first connection.
    let mut stream = TcpStream::connect("localhost:6788").await.unwrap();
    // Send a new connect packet
    let packet = Packet::Connect(Connect {
        client_id: Some("Suzuki".into()),
        ..Default::default()
    });
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    while let Err(_) = stream.write(&buffer).await {}

    // wait for receiving the Disconnect packet
    // If None, the operation was successful.
    if let Some(message) = wait_dis.await {
        panic!(message);
    }

    // End server
    println!("End server");
   // task::block_on(server.cancel());
}
