use crate::{Event, EventSender};
use async_std::{net::TcpListener, prelude::*, task};
use futures::SinkExt;
use log::{error, info};

pub async fn listen_tcp(listener: TcpListener, mut event_sender: EventSender) {
    // Listen to any connection
    info!(
        "Start listening to {:?} ({})",
        listener.local_addr(),
        task::current().id()
    );
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
                        .send(Event::NewPeer(stream, event_sender.clone()))
                        .await
                    {
                        error!("Cannot send event: {:?}", e);
                    }
                } else {
                    error!("Cannot get peer address");
                }
            }
        }
    }
    info!(
        "Stop listening to {:?} ({})",
        listener.local_addr(),
        task::current().id()
    );
}
