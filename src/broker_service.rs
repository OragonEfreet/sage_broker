use async_std::{
    future,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use log::{error, info};
use sage_mqtt::ControlPacket;
use std::time::Duration;

/// Result type returned by `BrokerService::listen`
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// The broker instance. The main component of the application.
pub struct BrokerService {
    pub(crate) timeout_delay: u16,
}

impl BrokerService {
    /// Listen to the given addresses and responds to the connexion requests.
    pub async fn listen<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        let addrs = addr.to_socket_addrs().await?;

        info!(
            "Listening to {}",
            addrs
                .map(|addr| addr.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );

        let listener = TcpListener::bind(addr).await?;

        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            match stream {
                Err(e) => {
                    error!("Cannot accept Tcp stream: {}", e.to_string());
                }
                Ok(stream) => {
                    let peer_addr = stream.peer_addr()?;
                    info!("Accepting from {}", peer_addr);
                    task::spawn(Self::connection(self.timeout_delay, stream));
                }
            }
        }

        Ok(())
    }

    async fn connection(timeout_delay: u16, mut stream: TcpStream) {
        let peer_addr = stream.peer_addr().unwrap();
        // let mut reader = BufReader::new(stream);

        let out_time = Duration::from_secs(timeout_delay as u64);

        if let Ok(_) = future::timeout(out_time, ControlPacket::decode(&mut stream)).await {
            info!("Will do something");
        } else {
            info!("Connexion time out");
        }

        // if let Ok(packet) = packet {
        //     debug!("Received {:?}", packet);
        // } else {
        //     error!("=================");
        //     error!("{:?}. Will close connection", packet);
        //     error!("=================");
        // }

        info!("Close {}", peer_addr);
    }
}
