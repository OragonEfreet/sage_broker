//! CONNECT Actions requirements consists in all [MQTT 3.1.4-x] conformances.
//! It also describes some elements from [MQTT 3.1.2-x].
use std::net::SocketAddr;
use tokio::{net::TcpStream, task};

use sage_broker::BrokerSettings;
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::Instant;
pub mod utils;
use utils::client::{DisPacket, Response};
pub use utils::*;

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.2-4: If a CONNECT packet is received with Clean Start is set to 1,
/// the Client and Server MUST discard any existing Session and start a new
/// Session.
#[tokio::test]
async fn mqtt_3_1_2_4() {
    let (sessions, server, local_addr, shutdown) =
        server::spawn(BrokerSettings::valid_default()).await;

    let client_id = String::from("Jaden");

    // First, we connect a client with a fixed client_id and wait for ACK
    let stream = mqtt_3_1_4_4_connect(&client_id, &local_addr, None).await;

    // Some checks on the state of the current database
    let session_id = {
        let sessions = sessions.read().unwrap();
        assert_eq!(sessions.len(), 1); // We have 1 client exactly
        let session = sessions.get(&client_id).unwrap();
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
        panic!("{}", what);
    }

    let sessions = sessions.read().unwrap();
    assert_eq!(sessions.len(), 1); // We have 1 client exactly
    let session = sessions.get(&client_id).unwrap();

    // Test: Client ID must be same but session id must be different
    assert_eq!(session.client_id(), client_id);
    assert_ne!(session_id, session.id());

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.2-5: If a CONNECT packet is received with Clean Start set to 0 and there is a Session associated
/// with the Client Identifier, the Server MUST resume communications with the Client based on
/// state from the existing Session.
#[tokio::test]
async fn mqtt_3_1_2_5() {
    let (sessions, server, local_addr, shutdown) =
        server::spawn(BrokerSettings::valid_default()).await;

    let client_id = String::from("Jaden");

    // First, we connect a client with a fixed id and wait for ACK
    let stream = mqtt_3_1_4_4_connect(&client_id, &local_addr, None).await;

    let session_id = {
        // Search db for the current connexion
        let sessions = sessions.read().unwrap();
        assert_eq!(sessions.len(), 1); // We have 1 client exactly
        let session = sessions.get(&client_id).unwrap();
        assert_eq!(session.client_id(), client_id);
        String::from(session.id())
    };

    // Let's do the same, forcing clean start to 0
    mqtt_3_1_4_4_connect(&client_id, &local_addr, Some(false)).await;

    // The first client must have been disconnected by the server
    let policy = DisPacket::Ignore(Some(ReasonCode::SessionTakenOver));
    if let Some(what) = client::wait_close(stream, policy).await {
        panic!("{}", what);
    }

    let sessions = sessions.read().unwrap();
    assert_eq!(sessions.len(), 1); // Because previous session was taken over
    let session = sessions.get(&client_id).unwrap();

    // Test: Client ID and session ID must be same
    assert_eq!(session.client_id(), client_id);
    assert_eq!(session_id, session.id());

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.2-6: If a CONNECT packet is received with Clean Start set to 0 and there is no Session associated
/// with the Client Identifier, the Server MUST create a new Session.
/// (implicitely: other sessions exist)
#[tokio::test]
async fn mqtt_3_1_2_6() {
    let (sessions, server, local_addr, shutdown) =
        server::spawn(BrokerSettings::valid_default()).await;

    let first_client_id = String::from("Jaden");
    let second_client_id = String::from("Jarod");

    // First, we connect a client with a fixed id and wait for ACK
    mqtt_3_1_4_4_connect(&first_client_id, &local_addr, None).await;

    let session_id = {
        // Search db for the current connexion
        let sessions = sessions.read().unwrap();
        assert_eq!(sessions.len(), 1); // We have 1 client exactly
        let session = sessions.get(&first_client_id).unwrap();
        assert_eq!(session.client_id(), first_client_id);
        String::from(session.id())
    };

    // Let's do the same, forcing clean start to 0
    mqtt_3_1_4_4_connect(&second_client_id, &local_addr, Some(false)).await;

    let sessions = sessions.read().unwrap();
    assert_eq!(sessions.len(), 2); // We have 2 client exactly
    let session = sessions.get(&second_client_id).unwrap();

    // Test: Client ID must be same but session id must be different
    assert_eq!(session.client_id(), second_client_id);
    assert_ne!(session.client_id(), first_client_id);
    assert_ne!(session.id(), session_id);

    server::stop(shutdown, server).await;
}

///////////////////////////////////////////////////////////////////////////////
/// If the Server does not receive a CONNECT packet within a reasonable amount
/// of time after the Network Connection is established
/// the Server SHOULD close the Network Connection.
#[tokio::test]
async fn connect_timeout() {
    let timeout_delay = 1;
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: timeout_delay,
        ..BrokerSettings::valid_default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await;

    let now = Instant::now();
    let delay_with_tolerance = (timeout_delay as f32 * 1.5) as u64;

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
/// MQTT-3.1.4-1: The Server MUST validate that the CONNECT packet matches the
/// format described in section 3.1 and close the Network Connection if it does
/// not match.
/// NOTE we may want to add new invalidate scenarios here.
/// Cases that would send back a connack packet
#[tokio::test]
async fn mqtt_3_1_4_1() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await;

    // Send an invalid connect packet and wait for an immediate disconnection
    // from the server.
    let mut buffer = Vec::new();
    Packet::Connect(Default::default())
        .encode(&mut buffer)
        .await
        .unwrap();
    buffer[0] |= 0b1111; // Invalidate the packet

    // We test for an actual disconnection (None) from the server
    // because no connection was made, thus no disconnect packet
    // neither connack packet can be returned
    let response = client::send_waitback_data(&mut stream, buffer).await;
    println!("====> {:?}", response);
    assert!(matches!(response, Response::Close));
    //    if let Some(packet) = client::send_waitback_data(&mut stream, buffer).await {
    //        assert!(matches!(packet, Packet::Disconnect(_)));
    //        if let Packet::Disconnect(packet) = packet {
    //            assert_eq!(packet.reason_code, ReasonCode::MalformedPacket);
    //        }
    //    }

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
#[tokio::test]
async fn mqtt_3_1_4_2() {
    // Create a set of pairs with a server and an unsupported connect packet.
    // Each one will be tested against an expected ConnAck > 0x80
    let settings = BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
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
        let mut stream = client::spawn(&local_addr).await;

        // Send an unsupported connect packet and wait for an immediate disconnection
        // from the server.
        if let Response::Packet(packet) = client::send_waitback(&mut stream, connect.into()).await {
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
#[tokio::test]
async fn mqtt_3_1_4_3() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;

    let connect = Connect {
        client_id: Some("Suzuki".into()),
        ..Default::default()
    };

    ////////////////////////////////////////////////////////////////////////////
    // Using the first client, we send a Connect packet and wait for the ConnAck
    let (stream, _) = client::connect(&local_addr, connect.clone()).await;

    ////////////////////////////////////////////////////////////////////////////
    // We spawn a new task which uses the first connection to wait for a
    // Disconnect(SessionTakenOver) packet within the next ten seconds.
    let policy = DisPacket::Ignore(Some(ReasonCode::SessionTakenOver));
    let wait_dis = task::spawn(client::wait_close(stream, policy));

    ////////////////////////////////////////////////////////////////////////////
    // Meanwhile, we connect with a new connexion and the same client. This
    // must generate a Disconnect packet recevied by the first connection.
    client::connect(&local_addr, connect).await;

    // wait for receiving the Disconnect packet
    // If None, the operation was successful.
    if let Some(message) = wait_dis.await.unwrap() {
        panic!("{}", message);
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
    let mut stream = client::spawn(&local_addr).await;
    if let Response::Packet(Packet::ConnAck(packet)) =
        client::send_waitback(&mut stream, connect.into()).await
    {
        assert_eq!(packet.reason_code, ReasonCode::Success);
        assert!(packet.assigned_client_id.is_none());
    } else {
        panic!("Invalid packet type sent after Connect");
    }
    stream
}

///////////////////////////////////////////////////////////////////////////////
// MQTT-3.1.4-5: The Server MUST acknowledge the CONNECT packet with a CONNACK
// packet containing a 0x00 (Success) Reason Code.
#[tokio::test]
async fn mqtt_3_1_4_5() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;
    let mut stream = client::spawn(&local_addr).await;

    // Send an valid connect packet and wait for ACK 0
    if let Response::Packet(packet) =
        client::send_waitback(&mut stream, Packet::Connect(Default::default())).await
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

///////////////////////////////////////////////////////////////////////////////
/// MQTT-3.1.4-6: If the Server rejects the CONNECT, it MUST NOT process any
/// data sent by the Client after the CONNECT packet except AUTH packets.
/// NOTE: Currently AUTH packet is not accepted either
#[tokio::test]
async fn mqtt_3_1_4_6() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;

    // Let's create a collection of all packet types
    let packets = vec![
        Packet::Connect(Default::default()),
        Packet::ConnAck(Default::default()),
        Packet::Publish(Default::default()),
        Packet::PubAck(Default::default()),
        Packet::PubRec(Default::default()),
        Packet::PubRel(Default::default()),
        Packet::PubComp(Default::default()),
        Packet::Subscribe(Default::default()),
        Packet::SubAck(Default::default()),
        Packet::UnSubscribe(Default::default()),
        Packet::UnSubAck(Default::default()),
        Packet::PingReq,
        Packet::PingResp,
        Packet::Disconnect(Default::default()),
        Packet::Auth(Default::default()),
    ];
    for second_packet in packets {
        let mut stream = client::spawn(&local_addr).await;

        // Auth not supported, so we can reject a packet by providing one
        let rejected_connect = Connect {
            authentication: Some(Default::default()),
            ..Default::default()
        };

        // Send an rejected connect packet
        if let Response::Packet(packet) =
            client::send_waitback(&mut stream, Packet::Connect(rejected_connect)).await
        {
            assert!(matches!(packet, Packet::ConnAck(_)));
            if let Packet::ConnAck(packet) = packet {
                // We just want ack to be none of ok or malformed
                assert_ne!(packet.reason_code, ReasonCode::Success);
                assert_ne!(packet.reason_code, ReasonCode::MalformedPacket);
            }
        }

        // Immediately send a second packet and expect a disconnected stream
        assert!(matches!(
            client::send_waitback(&mut stream, second_packet).await,
            Response::Close
        ));
    }

    server::stop(shutdown, server).await;
}
