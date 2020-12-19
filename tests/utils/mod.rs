use async_std::{
    io::{self, prelude::*, Cursor, ErrorKind},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    task::{self, JoinHandle},
};
use futures::channel::mpsc;
use sage_broker::{service, Broker, BrokerSettings};
use sage_mqtt::Packet;
use std::time::Duration;

pub struct TestServer {
    pub broker: Arc<Broker>,
    service_task: JoinHandle<()>,
    pub local_addr: SocketAddr,
}
pub const TIMEOUT_DELAY: u16 = 3;

impl TestServer {
    pub async fn prepare(settings: BrokerSettings) -> TestServer {
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();

        let broker = Broker::build(settings);
        let service_task = task::spawn(run_server(listener, broker.clone()));

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

/// Sends the given packet and wait for the next response from the server.
/// Note that nothing ensures the received packet from the server is a response
/// to the sent packet.
pub async fn send_waitback(
    stream: &mut TcpStream,
    packet: Packet,
    invalid: bool,
) -> Option<Packet> {
    // Send the packet as buffer
    let mut buffer = Vec::new();
    packet.encode(&mut buffer).await.unwrap();

    if invalid {
        buffer[0] |= 0b1111; // Invalidate the packet
                             // TODO Assert the packet IS invalid
    }
    while stream.write(&buffer).await.is_err() {} // TODO Should we timeout this?

    let delay_with_tolerance = Duration::from_secs((TIMEOUT_DELAY as f32 * 1.5) as u64);
    let mut buf = vec![0u8; 1024];
    match io::timeout(delay_with_tolerance, stream.read(&mut buf)).await {
        Err(e) => match e.kind() {
            ErrorKind::TimedOut => panic!("Server did not respond to invalid connect packet"),
            _ => panic!("IO Error: {:?}", e),
        },
        Ok(0) => None,
        Ok(_) => {
            let mut buf = Cursor::new(buf);
            Some(Packet::decode(&mut buf).await.unwrap())
        }
    }
}

// Run a server instance
async fn run_server(listener: TcpListener, broker: Arc<Broker>) {
    let (control_sender, control_receiver) = mpsc::unbounded();
    let control_loop = task::spawn(service::control_loop(broker.clone(), control_receiver));
    service::listen_tcp(listener, control_sender, broker.clone()).await;
    control_loop.await;
    broker.wait_pending().await;
}
