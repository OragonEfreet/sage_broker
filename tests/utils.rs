use async_std::{
    net::{SocketAddr, TcpStream},
    sync::Arc,
    task::{self, JoinHandle},
};
use sage_broker::{service, Broker, BrokerSettings};
use std::time::Duration;

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
