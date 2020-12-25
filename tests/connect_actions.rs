use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    task,
};
use sage_broker::BrokerSettings;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::{Duration, Instant};

mod utils;
use utils::{client, server, TIMEOUT_DELAY};

///////////////////////////////////////////////////////////////////////////////
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
#[async_std::test]
async fn connect_timeout() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await.unwrap();

    let now = Instant::now();
    let delay_with_tolerance = (utils::TIMEOUT_DELAY as f32 * 1.5) as u64;

    if let Ok(packet) = Packet::decode(&mut stream).await {
        match packet {
            Packet::Disconnect(packet) => {
                assert_eq!(packet.reason_code, ReasonCode::KeepAliveTimeout)
            }
            _ => panic!("Server must send Disconnect packet only"),
        }
    }

    assert!(now.elapsed().as_secs() >= delay_with_tolerance);
    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
#[async_std::test]
async fn mqtt_3_1_4_1() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: utils::TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await.unwrap();

    // Send an invalid connect packet and wait for an immediate disconnection
    // from the server.
    if let Some(packet) =
        client::send_waitback(&mut stream, Packet::Connect(Default::default()), true).await
    {
        assert!(matches!(packet, Packet::ConnAck(_)));
        if let Packet::ConnAck(packet) = packet {
            assert_eq!(packet.reason_code, ReasonCode::MalformedPacket);
        }
    }

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
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
        keep_alive: utils::TIMEOUT_DELAY,
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
    ];

    for (settings, connect, reason_code) in test_collection {
        let (_, server, local_addr, shutdown) = server::spawn(settings).await;
        let mut stream = client::spawn(&local_addr).await.unwrap();

        // Send an unsupported connect packet and wait for an immediate disconnection
        // from the server.
        if let Some(packet) = client::send_waitback(&mut stream, connect.into(), false).await {
            if let Packet::ConnAck(packet) = packet {
                assert_eq!(packet.reason_code, reason_code);
            } else {
                panic!("Packet should be ConnAck");
            }
        }
        server::stop(shutdown, server).await;
    }
}

///////////////////////////////////////////////////////////////////////////////
/// If the ClientID represents a Client already connected to the Server, the
/// Server sends a DISCONNECT packet to the existing Client with Reason Code
/// of 0x8E (Session taken over) as described in section 4.13 and MUST close
/// the Network Connection of the existing Client [MQTT-3.1.4-3]
#[async_std::test]
async fn mqtt_3_1_4_3() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: utils::TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await.unwrap();

    ////////////////////////////////////////////////////////////////////////////
    // Using the first client, we send a Connect packet and wait for the ConnAck
    let connect = Connect {
        client_id: Some("Suzuki".into()),
        ..Default::default()
    };

    // By unwrapping we ensure to panic! is the server disconnected
    if let Packet::ConnAck(packet) = client::send_waitback(&mut stream, connect.into(), false)
        .await
        .unwrap()
    {
        assert_eq!(packet.reason_code, ReasonCode::Success);
    } else {
        panic!("Invalid packet type sent after Connect");
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
    let mut stream = client::spawn(&local_addr).await.unwrap();
    // Send a new connect packet
    let packet = Packet::Connect(Connect {
        client_id: Some("Suzuki".into()),
        ..Default::default()
    });
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    while stream.write(&buffer).await.is_err() {}

    // wait for receiving the Disconnect packet
    // If None, the operation was successful.
    if let Some(message) = wait_dis.await {
        panic!(message);
    }

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// If a CONNECT packet is received with Clean Start is set to 1, the Client and Server MUST
/// discard any existing Session and start a new Session.
#[async_std::test]
async fn mqtt_3_1_2_4() {
    let (_, server, local_addr, shutdown) = server::spawn(Default::default()).await;
    let _stream = client::spawn(&local_addr).await.unwrap();
    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// If a CONNECT packet is received with Clean Start set to 0 and there is a Session associated
/// with the Client Identifier, the Server MUST resume communications with the Client based on
/// state from the existing Session.
#[async_std::test]
async fn mqtt_3_1_2_5() {
    let (_, server, local_addr, shutdown) = server::spawn(Default::default()).await;
    let _stream = client::spawn(&local_addr).await.unwrap();
    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// If a CONNECT packet is received with Clean Start set to 0 and there is no Session associated
/// with the Client Identifier, the Server MUST create a new Session.
#[async_std::test]
async fn mqtt_3_1_2_6() {
    let (_, server, local_addr, shutdown) = server::spawn(Default::default()).await;
    let _stream = client::spawn(&local_addr).await.unwrap();
    server::stop(shutdown, server).await;
}
///////////////////////////////////////////////////////////////////////////////
// The Server MUST perform the processing of Clean Start that is described in
// section 3.1.2.4 [MQTT-3.1.4-4]
// Aggregation of:
// - [MQTT-3.1.2-4]: mqtt_3_1_2_4()
// - [MQTT-3.1.2-5]: mqtt_3_1_2_5()
// - [MQTT-3.1.2-6]: mqtt_3_1_2_6()
