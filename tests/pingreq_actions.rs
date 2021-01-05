//! PINGREQ Actions requirements consists in all [MQTT 3.12.4-x] conformances.
use sage_broker::BrokerSettings;
use sage_mqtt::Packet;

pub mod utils;
use utils::client::Response;
use utils::*;

/// The Server MUST send a PINGRESP packet in response to a PINGREQ packet
/// [MQTT-3.12.4-1].
#[async_std::test]
async fn mqtt_3_12_4_1() {
    let (_, _, server, local_addr, shutdown) = server::spawn(BrokerSettings {
        ..Default::default()
    })
    .await;

    let (mut stream, _) = client::connect(&local_addr, Default::default()).await;

    assert!(matches!(
        client::send_waitback(&mut stream, Packet::PingReq).await,
        Response::Packet(Packet::PingResp)
    ));

    server::stop(shutdown, server).await;
}
