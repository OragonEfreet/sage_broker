use async_std::{
    io::{self, prelude::*, BufReader, ErrorKind},
    net::TcpStream,
    task::{self, JoinHandle},
};
use sage_broker::{Broker, BrokerConfig};
use sage_mqtt::Packet;
use std::time::{Duration, Instant};

const TIMEOUT_DELAY: u16 = 3;

async fn prepare_connection() -> (JoinHandle<()>, TcpStream) {
    let handle = {
        let config = &BrokerConfig::new("localhost:6788").with_connect_timeout_delay(TIMEOUT_DELAY);
        Broker::from_config(config).run()
    };

    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // pannic
    for _ in 0u8..5u8 {
        if let Ok(stream) = TcpStream::connect("localhost:6788").await {
            return (handle, stream);
        }

        task::sleep(Duration::from_secs(1)).await;
    }

    panic!("Cannot establish connection");
}

/// Requirements:
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
#[test]
fn connect_timeout() {
    let (server, stream) = task::block_on(prepare_connection());

    task::block_on(async {
        let mut stream = BufReader::new(stream);

        let now = Instant::now();
        let delay_with_tolerance = (TIMEOUT_DELAY as f32 * 1.5) as u64;

        // Listen loop and sending nothing
        loop {
            let mut recv_buffer = String::new();
            if let Ok(0) = stream.read_line(&mut recv_buffer).await {
                break;
            }

            assert!(now.elapsed().as_secs() <= delay_with_tolerance);
        }

        assert!(now.elapsed().as_secs() >= TIMEOUT_DELAY as u64);
    });

    task::block_on(server.cancel());
}

/// Requirements:
/// The Server MUST validate that the CONNECT packet matches the format
/// described in section 3.1 and close the Network Connection if it does not
/// match [MQTT-3.1.4-1].
/// The Server MAY send a CONNACK with a Reason Code of 0x80 or greater as
/// described in section 4.13 before closing the Network Connection.
#[test]
fn mqtt_3_1_4_1() {
    let (server, mut stream) = task::block_on(prepare_connection());

    task::block_on(async {
        // Send an invalid connect packet and wait for an immediate disconnection
        // from the server.
        let packet = Packet::Connect(Default::default());
        let mut buffer = Vec::new();
        packet.encode(&mut buffer).await.unwrap();

        while let Err(_) = stream.write_all(&buffer[..4]).await {}

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
            Ok(_) => panic!("Incomplete test. Should check for a valid CONNACK"),
        }
    });

    task::block_on(server.cancel());
}
