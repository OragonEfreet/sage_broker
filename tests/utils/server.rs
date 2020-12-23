use async_std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    task::{self, JoinHandle},
};
use futures::channel::mpsc;
use sage_broker::{service, BrokerSettings, Sessions, Trigger};
use std::time::Duration;

////////////////////////////////////////////////////////////////////////////////
/// Server instance used for testing purpose
pub struct TestServer {
    pub settings: Arc<BrokerSettings>,
    service_task: JoinHandle<()>,
    pub local_addr: SocketAddr,
    shutdown: Trigger,
}

impl TestServer {
    // Builds and run a server using the given settings
    pub async fn prepare(settings: BrokerSettings) -> TestServer {
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let settings = Arc::new(settings);

        let shutdown = Trigger::default();
        let service_task = task::spawn(run_server(listener, settings.clone(), shutdown.clone()));

        TestServer {
            settings,
            service_task,
            local_addr,
            shutdown,
        }
    }

    // Create a valid TcpStream client for the current server
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
        self.shutdown.fire().await;
        self.service_task.await;
    }
}

// Run a server instance
async fn run_server(listener: TcpListener, settings: Arc<BrokerSettings>, shutdown: Trigger) {
    let (command_sender, command_receiver) = mpsc::unbounded();
    let command_loop = task::spawn(service::command_loop(
        Sessions::default(),
        settings.clone(),
        command_receiver,
        shutdown.clone(),
    ));
    service::listen_tcp(listener, command_sender, settings, shutdown).await;
    command_loop.await;
}
