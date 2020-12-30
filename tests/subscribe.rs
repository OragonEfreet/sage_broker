//! SUBSCRIBE Actions requirements
use sage_broker::BrokerSettings;
use sage_mqtt::{Packet, ReasonCode};
pub mod utils;
pub use utils::*;

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.1-1: Bits 3,2,1 and 0 of the Fixed Header of the SUBSCRIBE packet are reserved and
/// MUST be set to 0,0,1 and 0 respectively. The Server MUST treat any other value as malformed and
/// close the Network Connection
#[async_std::test]
async fn mqtt_3_8_1_1() {
    // We will send any version of incorrect Fixed Headers and wait for the server's response
    // We will also check for the validity of the unique correct case
    let (_, _, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    })
    .await;
    let fixed_headers = vec![
        // SUBS_3210
        (0b1000_0000, ReasonCode::MalformedPacket),
        (0b1000_0001, ReasonCode::MalformedPacket),
        (0b1000_0010, ReasonCode::ProtocolError), // This one is valid
        (0b1000_0011, ReasonCode::MalformedPacket),
        (0b1000_0100, ReasonCode::MalformedPacket),
        (0b1000_0101, ReasonCode::MalformedPacket),
        (0b1000_0110, ReasonCode::MalformedPacket),
        (0b1000_0111, ReasonCode::MalformedPacket),
        (0b1000_1000, ReasonCode::MalformedPacket),
        (0b1000_1001, ReasonCode::MalformedPacket),
        (0b1000_1010, ReasonCode::MalformedPacket),
        (0b1000_1011, ReasonCode::MalformedPacket),
        (0b1000_1100, ReasonCode::MalformedPacket),
        (0b1000_1101, ReasonCode::MalformedPacket),
        (0b1000_1110, ReasonCode::MalformedPacket),
        (0b1000_1111, ReasonCode::MalformedPacket),
    ];

    for (fixed_header, expected) in fixed_headers {
        let mut stream = client::connect(&local_addr, Default::default()).await;

        let mut buffer = Vec::new();
        Packet::Subscribe(Default::default())
            .encode(&mut buffer)
            .await
            .unwrap();
        buffer[0] = fixed_header; // Invalidate the packet

        if let Some(packet) = client::send_waitback_data(&mut stream, buffer).await {
            if let Packet::ConnAck(packet) = packet {
                assert_eq!(packet.reason_code, expected);
            } else {
                panic!("Expected CONNACK after SUBSCRIBE");
            }
        }
    }

    server::stop(shutdown, server).await;
}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-1: The Topic Filters MUST be a UTF-8 Encoded String.
#[async_std::test]
async fn mqtt_3_8_3_1() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-2: The Payload MUST contain at least one Topic Filter and Subscription Options pair.
#[async_std::test]
async fn mqtt_3_8_3_2() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-3: Bit 2 of the Subscription Options represents the No Local option. If the value is
/// 1, Application Messages MUST NOT be forwarded to a connection with a ClientID equal to the
/// ClientID of the publishing connection.
#[async_std::test]
async fn mqtt_3_8_3_3() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-4: It is a Protocol Error to set the No Local bit to 1 on a Shared Subscription.
#[async_std::test]
async fn mqtt_3_8_3_4() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.3-5: The Server MUST treat a SUBSCRIBE packet as malformed if any of Reserved bits in
/// the Payload are non-zero.
#[async_std::test]
async fn mqtt_3_8_3_5() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-1: When the Server receives a SUBSCRIBE packet from a Client, the Server MUST
/// respond with a SUBACK packet.
#[async_std::test]
async fn mqtt_3_8_4_1() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-2: The SUBACK packet MUST have the same Packet Identifier as the SUBSCRIBE packet
/// that it is acknowledging.
#[async_std::test]
async fn mqtt_3_8_4_2() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-3: If a Server receives a SUBSCRIBE packet containing a Topic Filter that is
/// identical to a Non‑shared Subscription’s Topic Filter for the current Session then it MUST
/// replace that existing Subscription with a new Subscription.
#[async_std::test]
async fn mqtt_3_8_4_3() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-4: If the Retain Handling option is 0, any existing retained messages matching the
/// Topic Filter MUST be re-sent, but Application Messages MUST NOT be lost due to replacing the
/// Subscription.
#[async_std::test]
async fn mqtt_3_8_4_4() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-5: If a Server receives a SUBSCRIBE packet that contains multiple Topic Filters it
/// MUST handle that packet as if it had received a sequence of multiple SUBSCRIBE packets, except
/// that it combines their responses into a single SUBACK response.
#[async_std::test]
async fn mqtt_3_8_4_5() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-6: The SUBACK packet sent by the Server to the Client MUST contain a Reason Code for
/// each Topic Filter/Subscription Option pair.
#[async_std::test]
async fn mqtt_3_8_4_6() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-7: This Reason Code MUST either show the maximum QoS that was granted for that
/// Subscription or indicate that the subscription failed.
#[async_std::test]
async fn mqtt_3_8_4_7() {}

////////////////////////////////////////////////////////////////////////////////
/// MQTT-3.8.4-8: The QoS of Payload Messages sent in response to a Subscription MUST be the
/// minimum of the QoS of the originally published message and the Maximum QoS granted by the
/// Server.
#[async_std::test]
async fn mqtt_3_8_4_8() {}