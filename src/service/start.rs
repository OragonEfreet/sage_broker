use crate::{service, Broker, Event};
use async_std::{
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    sync::Arc,
    task::{self, JoinHandle},
};
use futures::{channel::mpsc, SinkExt};
use log::{error, info};

pub fn start(config: Broker) -> JoinHandle<()> {
    let (mut event_sender, event_receiver) = mpsc::unbounded();
    let config = Arc::new(config);

    task::spawn(service::event_loop(
        config.clone(),
        event_sender.clone(),
        event_receiver,
    ));

    let addr = config.addr.clone();

    task::spawn(async move {
        if let Ok(addrs) = addr.to_socket_addrs().await {
            info!(
                "Listening to {}",
                addrs
                    .map(|addr| addr.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            // Listen to any connection
            if let Ok(listener) = TcpListener::bind(addr).await {
                let mut incoming = listener.incoming();
                while let Some(stream) = incoming.next().await {
                    match stream {
                        Err(e) => {
                            error!("Cannot accept Tcp stream: {}", e.to_string());
                        }
                        Ok(stream) => {
                            if let Ok(peer_addr) = stream.peer_addr() {
                                info!("Accepting from {}", peer_addr);
                                if let Err(e) = event_sender.send(Event::NewPeer(stream)).await {
                                    error!("{:?}", e);
                                }
                            } else {
                                error!("Cannot get peer address");
                            }
                        }
                    }
                }
            } else {
                error!("Cannot listen socket");
            }
        } else {
            error!("Cannot compute socket addressed");
        }
    })
}
