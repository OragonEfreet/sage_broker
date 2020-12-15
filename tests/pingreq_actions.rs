use sage_broker::BrokerSettings;
use sage_mqtt::{Packet, ReasonCode};

mod utils;
use utils::TestServer;

/// The Server MUST send a PINGRESP packet in response to a PINGREQ packet
/// [MQTT-3.12.4-1].
#[async_std::test]
async fn mqtt_3_12_4_1() {
    let server = TestServer::prepare(BrokerSettings {
        ..Default::default()
    })
    .await;
    let mut stream = server.create_client().await.unwrap();

    // Send a valid connect packet and wait for connack
    // By unwrapping we ensure to panic! is the server disconnected
    if let Packet::ConnAck(packet) =
        utils::send_waitback(&mut stream, Packet::Connect(Default::default()), false)
            .await
            .unwrap()
    {
        assert_eq!(packet.reason_code, ReasonCode::Success);
    } else {
        panic!("Invalid packet type sent after Connect");
    }

    // Send a valid connect packet and wait for connack
    let packet = utils::send_waitback(&mut stream, Packet::PingReq, false)
        .await
        .unwrap();
    assert!(matches!(packet, Packet::PingResp));

    server.stop().await;
}
