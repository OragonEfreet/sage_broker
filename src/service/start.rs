use crate::{Broker, Event};
use async_std::{
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    sync::{Arc, RwLock},
    task::{self, JoinHandle},
};
use futures::SinkExt;
use log::{error, info};

pub fn start(broker: Broker) -> JoinHandle<()> {
    task::spawn(async move {
        let addr = broker.config.read().await.addr.clone();
        let mut event_sender = broker.event_sender.clone();
        let broker = Arc::new(broker);

        if let Ok(addrs) = addr.to_socket_addrs().await {
            // Listen to any connection
            if let Ok(listener) = TcpListener::bind(addr).await {
                let addrs = addrs
                    .map(|addr| addr.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                info!("Start listening to {} ({})", addrs, task::current().id());
                let mut incoming = listener.incoming();
                while let Some(stream) = incoming.next().await {
                    match stream {
                        Err(e) => {
                            error!("Cannot accept Tcp stream: {}", e.to_string());
                        }
                        Ok(stream) => {
                            if let Ok(peer_addr) = stream.peer_addr() {
                                info!("Incoming connection from {}", peer_addr);
                                if let Err(e) = event_sender
                                    .send(Event::NewPeer(broker.clone(), stream))
                                    .await
                                {
                                    error!("{:?}", e);
                                }
                            } else {
                                error!("Cannot get peer address");
                            }
                        }
                    }
                }
                info!("Stop listening to {} ({})", addrs, task::current().id());
            } else {
                error!("Cannot listen socket");
            }
        } else {
            error!("Cannot compute socket addressed");
        }
    })
}
