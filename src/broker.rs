use async_std::{
    future::{self, TimeoutError},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task,
};
use log::{debug, error, info};
use sage_mqtt::ControlPacket;
use std::time::Duration;

/// Result type returned by `Broker::listen`
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// The broker instance. The main component of the application.
pub struct Broker {
    pub(crate) connect_timeout_delay: u16,
}

impl Broker {
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
                    task::spawn(connection(stream));
                }
            }
        }

        Ok(())
    }
}

async fn connection(mut stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    // let mut reader = BufReader::new(stream);

    let out_time = Duration::from_secs(5);

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
