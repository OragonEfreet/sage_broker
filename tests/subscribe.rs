//! SUBSCRIBE Actions requirements
use sage_broker::BrokerSettings;
use sage_mqtt::{Packet, ReasonCode, Subscribe, Topic};
pub mod utils;

use utils::client::Response;
pub use utils::*;

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.1-1: Bits 3,2,1 and 0 of the Fixed Header of the SUBSCRIBE packet are reserved and
/// MUST be set to 0,0,1 and 0 respectively. The Server MUST treat any other value as malformed and
/// close the Network Connection
macro_rules! mqtt_3_8_1_1 {
    ($($name:ident: $value:expr,)*) => {
        $(

                #[tokio::test]
                async fn $name() {
                    let (fixed_header, expect_success) = $value;
                    mqtt_3_8_1_1(fixed_header, expect_success).await
                }


        )*
    }
}

mqtt_3_8_1_1! {
    mqtt_3_8_1_1_0010: (0b1000_0010, true),
    mqtt_3_8_1_1_0000: (0b1000_0000, false),
    mqtt_3_8_1_1_0001: (0b1000_0001, false),
    mqtt_3_8_1_1_0011: (0b1000_0011, false),
    mqtt_3_8_1_1_0100: (0b1000_0100, false),
    mqtt_3_8_1_1_0101: (0b1000_0101, false),
    mqtt_3_8_1_1_0110: (0b1000_0110, false),
    mqtt_3_8_1_1_0111: (0b1000_0111, false),
    mqtt_3_8_1_1_1000: (0b1000_1000, false),
    mqtt_3_8_1_1_1001: (0b1000_1001, false),
    mqtt_3_8_1_1_1010: (0b1000_1010, false),
    mqtt_3_8_1_1_1011: (0b1000_1011, false),
    mqtt_3_8_1_1_1100: (0b1000_1100, false),
    mqtt_3_8_1_1_1101: (0b1000_1101, false),
    mqtt_3_8_1_1_1110: (0b1000_1110, false),
    mqtt_3_8_1_1_1111: (0b1000_1111, false),
}

async fn mqtt_3_8_1_1(fixed_header: u8, expect_success: bool) {
    // We will send any version of incorrect Fixed Headers and wait for the server's response
    // We will also check for the validity of the unique correct case
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;

    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;

    let mut buffer = Vec::new();
    Packet::Subscribe(Subscribe {
        subscriptions: vec![(Topic::from("hello/world"), Default::default())],
        ..Default::default()
    })
    .encode(&mut buffer)
    .await
    .unwrap();
    buffer[0] = fixed_header; // Force Fixed Header value

    if expect_success {
        assert!(matches!(
            client::send_waitback_data(&mut stream, buffer).await,
            Response::Packet(Packet::SubAck(_))
        ));
    } else {
        let response = client::send_waitback_data(&mut stream, buffer).await;
        if let Response::Packet(packet) = response {
            if let Packet::Disconnect(packet) = packet {
                assert_eq!(packet.reason_code, ReasonCode::MalformedPacket);
            } else {
                panic!("Expected DISCONNECT after malformed SUBSCRIBE");
            }
        } else {
            panic!("Expected packet after malformed SUBSCRIBE");
        }
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-1: The Topic Filters MUST be a UTF-8 Encoded String.
#[tokio::test]
async fn mqtt_3_8_3_1() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-2: The Payload MUST contain at least one Topic Filter and Subscription Options pair.
/// A SUBSCRIBE packet with no Payload is a Protocol Error
#[tokio::test]
async fn mqtt_3_8_3_2() {
    // We will send a packet with no Subscription
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;

    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;

    let mut buffer = Vec::new();
    Packet::Subscribe(Default::default())
        .encode(&mut buffer)
        .await
        .unwrap();

    let response = client::send_waitback_data(&mut stream, buffer).await;
    if let Response::Packet(packet) = response {
        if let Packet::Disconnect(packet) = packet {
            assert_eq!(packet.reason_code, ReasonCode::ProtocolError);
        } else {
            panic!("Expected DISCONNECT after empty SUBSCRIBE");
        }
    } else {
        panic!("Expected packet after empty SUBSCRIBE");
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-3: Bit 2 of the Subscription Options represents the No Local option. If the value is
/// 1, Application Messages MUST NOT be forwarded to a connection with a ClientID equal to the
/// ClientID of the publishing connection.
#[tokio::test]
async fn mqtt_3_8_3_3() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-4: It is a Protocol Error to set the No Local bit to 1 on a Shared Subscription.
#[tokio::test]
async fn mqtt_3_8_3_4() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-5: The Server MUST treat a SUBSCRIBE packet as malformed if any of Reserved bits in
/// the Payload are non-zero.
#[tokio::test]
async fn mqtt_3_8_3_5_0001() {
    mqtt_3_8_3_5(0b0100_0000).await
}

#[tokio::test]
async fn mqtt_3_8_3_5_0010() {
    mqtt_3_8_3_5(0b1000_0000).await
}

#[tokio::test]
async fn mqtt_3_8_3_5_0011() {
    mqtt_3_8_3_5(0b1100_0000).await
}

async fn mqtt_3_8_3_5(sub_payload: u8) {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;

    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;
    let mut buffer = Vec::new();
    // Create a subscribe packet with a topic

    let subs = Subscribe {
        subscriptions: vec![("Cat".into(), Default::default())],
        ..Default::default()
    };
    Packet::Subscribe(subs).encode(&mut buffer).await.unwrap();
    *buffer.last_mut().unwrap() |= sub_payload; // Force Fixed Header value

    let response = client::send_waitback_data(&mut stream, buffer).await;
    if let Response::Packet(packet) = response {
        if let Packet::Disconnect(packet) = packet {
            assert_eq!(packet.reason_code, ReasonCode::MalformedPacket);
        } else {
            panic!(
                "Expected DISCONNECT after invalid SUBSCRIBE, got {:?}",
                packet
            );
        }
    } else {
        panic!("Expected packet after invalid SUBSCRIBE");
    }
    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-1: When the Server receives a SUBSCRIBE packet from a Client, the Server MUST
/// respond with a SUBACK packet.
#[tokio::test]
async fn mqtt_3_8_4_1() {
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;
    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;

    let subscribe = Subscribe {
        subscriptions: vec![(Topic::from("hello/world"), Default::default())],
        ..Default::default()
    };

    assert!(matches!(
        client::send_waitback(&mut stream, subscribe.into()).await,
        Response::Packet(Packet::SubAck(_))
    ));

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-2: The SUBACK packet MUST have the same Packet Identifier as the SUBSCRIBE packet
/// that it is acknowledging.
#[tokio::test]
async fn mqtt_3_8_4_2() {
    // We send 100 random packet identifiers and expect the SubAck to return the same each
    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;
    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;
    for _ in 0..100 {
        let packet_identifier = rand::random();

        let subscribe = Subscribe {
            packet_identifier,
            subscriptions: vec![(Topic::from("hello/world"), Default::default())],
            ..Default::default()
        };

        if let Response::Packet(Packet::SubAck(suback)) =
            client::send_waitback(&mut stream, subscribe.into()).await
        {
            assert_eq!(suback.packet_identifier, packet_identifier);
        } else {
            panic!("Expected SUBACK after SUBSCRIBE");
        }
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-3: If a Server receives a SUBSCRIBE packet containing a Topic Filter that is
/// identical to a Non‑shared Subscription’s Topic Filter for the current Session then it MUST
/// replace that existing Subscription with a new Subscription.
#[tokio::test]
async fn mqtt_3_8_4_3() {
    let topic = Topic::from("Topic1");
    let (sessions, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;
    let (mut stream, client_id) = client::connect(&local_addr, Default::default()).await;
    let session = sessions.read().unwrap().get(&client_id.unwrap()).unwrap();

    // Send twice the same topic sub. Each time check only 1 sub exist within
    // the client
    for _ in 0..2 {
        let subscribe = Subscribe {
            subscriptions: vec![(topic.clone(), Default::default())],
            ..Default::default()
        };

        assert!(matches!(
            client::send_waitback(&mut stream, subscribe.into()).await,
            Response::Packet(Packet::SubAck(_))
        ));

        {
            let subs = session.subs().read().unwrap();
            assert_eq!(subs.len(), 1);
            assert!(subs.has_filter(&topic,));
        }
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-4: If the Retain Handling option is 0, any existing retained messages matching the
/// Topic Filter MUST be re-sent, but Application Messages MUST NOT be lost due to replacing the
/// Subscription.
#[tokio::test]
async fn mqtt_3_8_4_4() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-5: If a Server receives a SUBSCRIBE packet that contains multiple Topic Filters it
/// MUST handle that packet as if it had received a sequence of multiple SUBSCRIBE packets, except
/// that it combines their responses into a single SUBACK response.
#[tokio::test]
async fn mqtt_3_8_4_5() {
    // Send a sub with three topics
    let topics = vec!["topic1", "topic2", "topic3"];

    let (sessions, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;

    let (mut stream, client_id) = client::connect(&local_addr, Default::default()).await;
    let session = sessions.read().unwrap().get(&client_id.unwrap()).unwrap();

    let subscribe = Subscribe {
        subscriptions: topics
            .iter()
            .map(|&t| (Topic::from(t), Default::default()))
            .collect(),
        ..Default::default()
    };

    assert!(matches!(
        client::send_waitback(&mut stream, subscribe.into()).await,
        Response::Packet(Packet::SubAck(_))
    ));

    // Check for the existence of the three subscriptions in the session
    {
        let subs = session.subs().read().unwrap();
        assert_eq!(subs.len(), 3);
        assert!(subs.has_filter(&Topic::from("topic1")));
        assert!(subs.has_filter(&Topic::from("topic2")));
        assert!(subs.has_filter(&Topic::from("topic3")));
    }

    // Now resend each separately and make the same checks
    for subscribe in topics.iter().map(|&x| Subscribe {
        subscriptions: vec![(Topic::from(x), Default::default())],
        ..Default::default()
    }) {
        assert!(matches!(
            client::send_waitback(&mut stream, subscribe.into()).await,
            Response::Packet(Packet::SubAck(_))
        ));
    }

    {
        let subs = session.subs().read().unwrap();
        assert_eq!(subs.len(), 3);
        assert!(subs.has_filter(&Topic::from("topic1")));
        assert!(subs.has_filter(&Topic::from("topic2")));
        assert!(subs.has_filter(&Topic::from("topic3")));
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-6: The SUBACK packet sent by the Server to the Client MUST contain a Reason Code for
/// each Topic Filter/Subscription Option pair.
#[tokio::test]
async fn mqtt_3_8_4_6() {
    // Send a sub with three topics
    let topics = vec!["topic1", "topic2", "topic3"];

    let (_, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..BrokerSettings::valid_default()
    })
    .await;
    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;

    let subscribe = Subscribe {
        subscriptions: topics
            .iter()
            .map(|&t| (Topic::from(t), Default::default()))
            .collect(),
        ..Default::default()
    };

    let response = client::send_waitback(&mut stream, subscribe.into()).await;

    if let Response::Packet(Packet::SubAck(suback)) = response {
        assert_eq!(suback.reason_codes.len(), 3);
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-7: This Reason Code MUST either show the maximum QoS that was granted for that
/// Subscription or indicate that the subscription failed.
#[tokio::test]
async fn mqtt_3_8_4_7() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-8: The QoS of Payload Messages sent in response to a Subscription MUST be the
/// minimum of the QoS of the originally published message and the Maximum QoS granted by the
/// Server.
#[tokio::test]
async fn mqtt_3_8_4_8() {}
