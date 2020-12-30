//! CONNECT Actions requirements consists in all [MQTT 3.1.4-x] conformances.
//! It also describes some elements from [MQTT 3.1.2-x].
use async_std::{
    io::prelude::*,
    net::{SocketAddr, TcpStream},
    task,
};

use sage_broker::BrokerSettings;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::Instant;
pub mod utils;
use utils::client::DisPacket;
pub use utils::*;

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.2-4: If a CONNECT packet is received with Clean Start is set to 1, the Client and Server MUST
/// discard any existing Session and start a new Session.
#[async_std::test]
async fn mqtt_3_1_2_4() {
    let (_, sessions, server, local_addr, shutdown) = server::spawn(Default::default()).await;

    let client_id = String::from("Jaden");

    // First, we connect a client with a fixed client_id and wait for ACK
    let stream = mqtt_3_1_4_4_connect(&client_id, &local_addr, None).await;

    // Some checks on the state of the current database
    let session_id = {
        let db = sessions.db.read().await;
        assert_eq!(db.len(), 1); // We have 1 client exactly
        let session = db[0].read().await;
        assert_eq!(session.client_id(), client_id);
        String::from(session.id())
    };

    // Let's do the same, forcing clean start to 1
    mqtt_3_1_4_4_connect(&client_id, &local_addr, Some(true)).await;

    // Wait for the first client to be closed by the server, which MUST
    // happen.
    // The server may send a disconnect packet, in that case the reason code
    // must be SessionTakenOver
    let policy = DisPacket::Ignore(Some(ReasonCode::SessionTakenOver));
    if let Some(what) = client::wait_close(stream, policy).await {
        panic!(what);
    }

    let db = sessions.db.read().await;
    assert_eq!(db.len(), 1); // Because previous session was taken over
    let session = db[0].read().await;

    // Test: Client ID must be same but session id must be different
    assert_eq!(session.client_id(), client_id); // This is were client ids are compared
    assert_ne!(session_id, session.id()); // This is were sessions are compared

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.2-5: If a CONNECT packet is received with Clean Start set to 0 and there is a Session associated
/// with the Client Identifier, the Server MUST resume communications with the Client based on
/// state from the existing Session.
#[async_std::test]
async fn mqtt_3_1_2_5() {
    let (_, sessions, server, local_addr, shutdown) = server::spawn(Default::default()).await;

    let client_id = String::from("Jaden");

    // First, we connect a client with a fixed id and wait for ACK
    let stream = mqtt_3_1_4_4_connect(&client_id, &local_addr, None).await;

    let session_id = {
        // Search db for the current connexion
        let db = sessions.db.read().await;
        assert_eq!(db.len(), 1);
        let session = db[0].read().await;
        assert_eq!(session.client_id(), client_id);
        String::from(session.id())
    };

    // Let's do the same, forcing clean start to 0
    mqtt_3_1_4_4_connect(&client_id, &local_addr, Some(false)).await;

    // The first client must have been disconnected by the server
    let policy = DisPacket::Ignore(Some(ReasonCode::SessionTakenOver));
    if let Some(what) = client::wait_close(stream, policy).await {
        panic!(what);
    }

    let db = sessions.db.read().await;
    assert_eq!(db.len(), 1); // Because previous session was taken over
    let session = db[0].read().await;

    // Test: Client ID and session ID must be same
    assert_eq!(session.client_id(), client_id); // This is were client ids are compared
    assert_eq!(session_id, session.id()); // This is were sessions are compared

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.2-6: If a CONNECT packet is received with Clean Start set to 0 and there is no Session associated
/// with the Client Identifier, the Server MUST create a new Session.
/// (implicitely: other sessions exist)
#[async_std::test]
async fn mqtt_3_1_2_6() {
    let (_, sessions, server, local_addr, shutdown) = server::spawn(Default::default()).await;

    let first_client_id = String::from("Jaden");
    let second_client_id = String::from("Jarod");

    // First, we connect a client with a fixed id and wait for ACK
    mqtt_3_1_4_4_connect(&first_client_id, &local_addr, None).await;

    let session_id = {
        // Search db for the current connexion
        let db = sessions.db.read().await;
        assert_eq!(db.len(), 1);
        let session = db[0].read().await;
        assert_eq!(session.client_id(), first_client_id);
        String::from(session.id())
    };

    // Let's do the same, forcing clean start to 0
    mqtt_3_1_4_4_connect(&second_client_id, &local_addr, Some(false)).await;

    let db = sessions.db.read().await;
    assert_eq!(db.len(), 2); // Because former session has different ID
    let session = db[1].read().await; // The last created session is at 1

    // Test: Client ID must be same but session id must be different
    assert_eq!(session.client_id(), second_client_id);
    assert_ne!(session.client_id(), first_client_id); // New client's ID is not same as first one
    assert_ne!(session.id(), session_id); // Session ids are not the same

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// If the Server does not receive a CONNECT packet within a reasonable amount
/// of time after the Network Connection is established
/// the Server SHOULD close the Network Connection.
#[async_std::test]
async fn connect_timeout() {
    let (_, _, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await.unwrap();

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
    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.4-1: The Server MUST validate that the CONNECT packet matches the format described in
/// section 3.1 and close the Network Connection if it does not match.
#[async_std::test]
async fn mqtt_3_1_4_1() {
    let (_, _, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
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
        keep_alive: TIMEOUT_DELAY,
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
        let (_, _, server, local_addr, shutdown) = server::spawn(settings).await;
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
    let (_, _, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
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
    let policy = DisPacket::Ignore(Some(ReasonCode::SessionTakenOver));
    let wait_dis = task::spawn(client::wait_close(stream, policy));

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
// The Server MUST perform the processing of Clean Start that is described in
// section 3.1.2.4 [MQTT-3.1.4-4]
// Aggregation of:
// - [MQTT-3.1.2-4]: mqtt_3_1_2_4()
// - [MQTT-3.1.2-5]: mqtt_3_1_2_5()
// - [MQTT-3.1.2-6]: mqtt_3_1_2_6()
/// This function is used by all three functions to create a client, connect to
/// it with a given Clean Start option
/// It is not a test by itself
async fn mqtt_3_1_4_4_connect(
    client_id: &str,
    local_addr: &SocketAddr,
    clean_start: Option<bool>,
) -> TcpStream {
    let connect = if let Some(clean_start) = clean_start {
        Connect {
            client_id: Some(client_id.into()),
            clean_start,
            ..Default::default()
        }
    } else {
        Connect {
            client_id: Some(client_id.into()),
            ..Default::default()
        }
    };

    // First, we connect a client with a fixed id and wait for ACK
    let mut stream = client::spawn(&local_addr).await.unwrap();
    if let Packet::ConnAck(packet) = client::send_waitback(&mut stream, connect.into(), false)
        .await
        .unwrap()
    {
        assert_eq!(packet.reason_code, ReasonCode::Success);
        assert!(packet.assigned_client_id.is_none());
    } else {
        panic!("Invalid packet type sent after Connect");
    }
    stream
}

///////////////////////////////////////////////////////////////////////////////
// MQTT-3.1.4-5: The Server MUST acknowledge the CONNECT packet with a CONNACK packet containing
// a 0x00 (Success) Reason Code.
#[async_std::test]
async fn mqtt_3_1_4_5() {
    let (_, _, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await.unwrap();

    // Send an valid connect packet and wait for ACK 0
    if let Some(packet) =
        client::send_waitback(&mut stream, Packet::Connect(Default::default()), false).await
    {
        if let Packet::ConnAck(packet) = packet {
            assert_eq!(packet.reason_code, ReasonCode::Success);
        } else {
            panic!("Expected CONNACK(Success) packet after CONNECT");
        }
    } else {
        panic!("Expected packet after CONNECT");
    }

    server::stop(shutdown, server).await;
}
