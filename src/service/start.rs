use crate::{service, Broker, Event};
use async_std::{
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    sync::{Arc, RwLock},
    task::{self, JoinHandle},
};
use futures::channel::mpsc;
use futures::SinkExt;
use log::{error, info};

pub fn start(config: Broker) -> JoinHandle<()> {
    let (mut event_sender, event_receiver) = mpsc::unbounded();
    let config = Arc::new(RwLock::new(config));

    // TODO The envent loop should be waitable
    task::spawn(service::event_loop(
        config.clone(),
        event_sender.clone(),
        event_receiver,
    ));

    task::spawn(async move {
        let addr = config.read().await.addr.clone();

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

                                if let Err(e) = event_sender.send(Event::NewPeer(stream)).await {
                                    error!("Cannot send event: {:?}", e);
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
