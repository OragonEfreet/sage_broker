use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    task,
};
use sage_broker::BrokerSettings;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::{Duration, Instant};

mod utils;
use utils::TestServer;

const TIMEOUT_DELAY: u16 = 3;

/// Requirements:
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
#[async_std::test]
async fn connect_timeout() {
    let server = TestServer::prepare(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = server.create_client().await.unwrap();

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
    server.stop().await;
}

/// Requirements:
/// The Server MUST validate that the CONNECT packet matches the format
/// described in section 3.1 and close the Network Connection if it does not
/// match [MQTT-3.1.4-1].
/// The Server MAY send a CONNACK with a Reason Code of 0x80 or greater as
/// described in section 4.13 before closing the Network Connection.
#[async_std::test]
async fn mqtt_3_1_4_1() {
    let server = TestServer::prepare(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = server.create_client().await.unwrap();

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

    server.stop().await;
}

/// The Server MAY check that the contents of the CONNECT packet meet any
/// further restrictions and SHOULD perform authentication and authorization
/// checks. If any of these checks fail, it MUST close the Network Connection
/// [MQTT-3.1.4-2].
/// Before closing the Network Connection, it MAY send an appropriate CONNACK
/// response with a Reason Code of 0x80 or greater as described in section 3.2
/// and section 4.13.
#[async_std::test]
async fn mqtt_3_1_4_2() {
    // Create a set of pairs with a server and an unsupported connect packet.
    // Each one will be tested against an expected ConnAck > 0x80
    let settings = BrokerSettings {
        ..Default::default()
    };
    // Vector of (input,output) tests.
    // For each set:
    // - The firs telement is the setting to build the broker with
    // - The second element is the connect packet to be sent
    // - The last one is the expected ReasonCode
    let test_collection = vec![
        (
            settings.clone(),
            Connect {
                authentication: Some(Default::default()),
                ..Default::default()
            },
            ReasonCode::BadAuthenticationMethod,
        ),
        (
            settings.clone(),
            Connect {
                user_name: Some("Thanos".into()),
                ..Default::default()
            },
            ReasonCode::BadAuthenticationMethod,
        ),
        // (settings.clone(), Connect::default()), <-- This one must fail
    ];

    for (settings, connect, reason_code) in test_collection {
        let server = TestServer::prepare(settings).await;
        let mut stream = server.create_client().await.unwrap();

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
                ErrorKind::TimedOut => panic!("Server did not respond to invalid connect packet"),
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

        server.stop().await;
    }
}

/// If the ClientID represents a Client already connected to the Server, the
/// Server sends a DISCONNECT packet to the existing Client with Reason Code
/// of 0x8E (Session taken over) as described in section 4.13 and MUST close
/// the Network Connection of the existing Client [MQTT-3.1.4-3]
#[async_std::test]
async fn mqtt_3_1_4_3() {
    let server = TestServer::prepare(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = server.create_client().await.unwrap();

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
    // Disconnect(SessionTakenOver) packet within the next ten seconds.
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
    let mut stream = server.create_client().await.unwrap();
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

    server.stop().await;
}
