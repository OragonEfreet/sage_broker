use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::TcpStream,
    task,
};
use async_std::{net::SocketAddr, sync::Arc, task::JoinHandle};
use sage_broker::{service, Broker, BrokerSettings};
use sage_mqtt::{Connect, Packet, ReasonCode};
use std::time::Duration;

const TIMEOUT_DELAY: u16 = 3;

async fn test_function() {
    let settings = BrokerSettings {
        keep_alive: TIMEOUT_DELAY,
        ..Default::default()
    };

    let connect = Connect {
        authentication: Some(Default::default()),
        ..Default::default()
    };
    let reason_code = ReasonCode::BadAuthenticationMethod;

    let server = TestServer::prepare(settings).await;
    let mut stream = server.create_client().await.unwrap();

    // Send an invalid connect packet and wait for an immediate disconnection
    // from the server.
    let packet = Packet::from(connect);
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();
    while stream.write(&buffer).await.is_err() {}

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
        Ok(_) => {
            let mut buf = Cursor::new(buf);
            let packet = Packet::decode(&mut buf).await.unwrap();
            if let Packet::ConnAck(packet) = packet {
                assert_eq!(packet.reason_code, reason_code);
            } else {
                panic!("Packet should be ConnAck");
            }
        }
    }

    server.stop().await;
}

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    test_function().await;
}

pub struct TestServer {
    pub broker: Arc<Broker>,
    service_task: JoinHandle<()>,
    pub local_addr: SocketAddr,
}

impl TestServer {
    pub async fn prepare(settings: BrokerSettings) -> TestServer {
        let addr = "localhost:0";
        let listener = service::bind(addr).await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let broker = Broker::build(settings);
        let service_task = task::spawn(service::run(listener, broker.clone()));

        TestServer {
            broker,
            service_task,
            local_addr,
        }
    }

    pub async fn create_client(&self) -> Option<TcpStream> {
        // Makes 5 connexion attemps, every 1 second until a connexion is made, or
        // panic
        for _ in 0u8..5u8 {
            if let Ok(stream) = TcpStream::connect(&self.local_addr).await {
                return Some(stream);
            }

            task::sleep(Duration::from_secs(1)).await;
        }

        None
    }

    pub async fn stop(self) {
        self.broker.shutdown().await;
        self.service_task.await;
    }
}
