use async_std::{io::prelude::*, task};
use sage_broker::BrokerSettings;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::Instant;

pub mod utils;
pub use utils::*;

///////////////////////////////////////////////////////////////////////////////
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
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
    let wait_dis = task::spawn(client::wait_close(
        stream,
        client::DisconnectPolicy::Ignore(Some(ReasonCode::SessionTakenOver)),
    ));

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
    let (_, sessions, server, local_addr, shutdown) = server::spawn(Default::default()).await;

    let client_id = String::from("Jaden");

    // First, we connect a client with a fixed id and wait for ACK
    let mut stream = client::spawn(&local_addr).await.unwrap();
    let session_id = {
        if let Packet::ConnAck(packet) = client::send_waitback(
            &mut stream,
            Connect {
                client_id: Some(client_id.clone()),
                ..Default::default()
            }
            .into(),
            false,
        )
        .await
        .unwrap()
        {
            assert_eq!(packet.reason_code, ReasonCode::Success);
            assert!(packet.assigned_client_id.is_none());
        } else {
            panic!("Invalid packet type sent after Connect");
        }

        // Search db for the current connexion
        let db = sessions.db.read().await;
        assert_eq!(db.len(), 1);
        let session = db[0].read().await;
        assert_eq!(session.client_id(), client_id);
        String::from(session.client_id())
    };

    // Let's do the same, forcing clean start to 1
    let mut new_stream = client::spawn(&local_addr).await.unwrap();

    if let Packet::ConnAck(packet) = client::send_waitback(
        &mut new_stream,
        Connect {
            client_id: Some(client_id.clone()),
            clean_start: true, // Set Clean Start to 1. Session MUST be new
            ..Default::default()
        }
        .into(),
        false,
    )
    .await
    .unwrap()
    {
        assert_eq!(packet.reason_code, ReasonCode::Success);
        assert!(packet.assigned_client_id.is_none());
    } else {
        panic!("Invalid packet type sent after Connect");
    }

    // The first client must have been disconnected by the server
    if let Some(what) = client::wait_close(
        stream,
        client::DisconnectPolicy::Force(Some(ReasonCode::SessionTakenOver)),
    )
    .await
    {
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
/// If a CONNECT packet is received with Clean Start set to 0 and there is a Session associated
/// with the Client Identifier, the Server MUST resume communications with the Client based on
/// state from the existing Session.
#[async_std::test]
async fn mqtt_3_1_2_5() {
    let (_, sessions, server, local_addr, shutdown) = server::spawn(Default::default()).await;

    let client_id = String::from("Jaden");

    // First, we connect a client with a fixed id and wait for ACK
    let mut stream = client::spawn(&local_addr).await.unwrap();
    let session_id = {
        if let Packet::ConnAck(packet) = client::send_waitback(
            &mut stream,
            Connect {
                client_id: Some(client_id.clone()),
                ..Default::default()
            }
            .into(),
            false,
        )
        .await
        .unwrap()
        {
            assert_eq!(packet.reason_code, ReasonCode::Success);
            assert!(packet.assigned_client_id.is_none());
        } else {
            panic!("Invalid packet type sent after Connect");
        }

        // Search db for the current connexion
        let db = sessions.db.read().await;
        assert_eq!(db.len(), 1);
        let session = db[0].read().await;
        assert_eq!(session.client_id(), client_id);
        String::from(session.client_id())
    };

    // Let's do the same, forcing clean start to 1
    let mut new_stream = client::spawn(&local_addr).await.unwrap();

    if let Packet::ConnAck(packet) = client::send_waitback(
        &mut new_stream,
        Connect {
            client_id: Some(client_id.clone()),
            clean_start: false, // Set Clean Start to 0. Session MUST be kept
            ..Default::default()
        }
        .into(),
        false,
    )
    .await
    .unwrap()
    {
        assert_eq!(packet.reason_code, ReasonCode::Success);
        assert!(packet.assigned_client_id.is_none());
    } else {
        panic!("Invalid packet type sent after Connect");
    }

    // The first client must have been disconnected by the server
    if let Some(what) = client::wait_close(
        stream,
        client::DisconnectPolicy::Force(Some(ReasonCode::SessionTakenOver)),
    )
    .await
    {
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
/// If a CONNECT packet is received with Clean Start set to 0 and there is no Session associated
/// with the Client Identifier, the Server MUST create a new Session.
#[async_std::test]
async fn mqtt_3_1_2_6() {
    let (_, _, server, local_addr, shutdown) = server::spawn(Default::default()).await;
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
