use async_std::io::{self, prelude::*, Cursor};
use sage_broker::BrokerSettings;
use sage_mqtt::{Packet, ReasonCode};
use std::time::Duration;

mod utils;
use utils::TestServer;

const TIMEOUT_DELAY: u16 = 3;

/// The Server MUST send a PINGRESP packet in response to a PINGREQ packet
/// [MQTT-3.12.4-1].
#[async_std::test]
async fn mqtt_3_12_4_1() {
    let server = TestServer::prepare(BrokerSettings {
        ..Default::default()
    })
    .await;
    let mut stream = server.create_client().await.unwrap();

    // Send an valid connect packet and wait for connack
    let packet = Packet::Connect(Default::default());
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    while stream.write(&buffer).await.is_err() {}

    // Wait for a response from the server within the next seconds
    let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
    let mut buf = vec![0u8; 1024];
    let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

    if result.is_ok() {
        let mut buf = Cursor::new(buf);
        let packet = Packet::decode(&mut buf).await.unwrap();
        if let Packet::ConnAck(packet) = packet {
            assert_eq!(packet.reason_code, ReasonCode::Success);
        } else {
            panic!("Invalid packet type sent after Connect");
        }
    } else {
        panic!("Server did not respond correctly to connect packet");
    }

    // Send a ping request packet
    let packet = Packet::PingReq;
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    while stream.write(&buffer).await.is_err() {}

    // Wait for a response from the server within the next seconds
    let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
    let mut buf = vec![0u8; 1024];
    let result = io::timeout(delay_with_tolerance, stream.read(&mut buf)).await;

    if result.is_ok() {
        let mut buf = Cursor::new(buf);
        let packet = Packet::decode(&mut buf).await.unwrap();
        assert!(matches!(packet, Packet::PingResp));
    } else {
        panic!("Server did not respond correctly to connect packet");
    }

    server.stop().await;
}
