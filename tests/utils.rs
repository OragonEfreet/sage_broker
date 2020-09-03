use async_std::{
    net::{SocketAddr, TcpStream},
    task::{self, JoinHandle},
};
use sage_broker::{service, BrokerSettings};
use std::time::Duration;

pub async fn setup(
    settings: BrokerSettings,
    addr: &str,
) -> Option<(JoinHandle<()>, TcpStream, SocketAddr)> {
    let listener = service::bind(addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    println!("{:?}", local_addr);
    let handle = task::spawn(service::start(listener, settings.into()));

    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // panic
    for _ in 0u8..5u8 {
        if let Ok(stream) = TcpStream::connect(&addr).await {
            return Some((handle, stream, local_addr));
        }

        task::sleep(Duration::from_secs(1)).await;
    }

    None
}
