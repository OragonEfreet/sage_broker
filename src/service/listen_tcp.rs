use crate::{service, Broker, ControlSender, Peer};
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use log::{error, info};

pub async fn listen_tcp(listener: TcpListener, control_sender: ControlSender, broker: Arc<Broker>) {
    // Listen to any connection
    info!(
        "Start listening to {:?} ({})",
        listener.local_addr(),
        task::current().id()
    );
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        match stream {
            Err(e) => error!("Cannot accept Tcp stream: {}", e.to_string()),
            Ok(stream) => create_peer(stream, control_sender.clone(), &broker).await,
        }
    }
    info!(
        "Stop listening to {:?} ({})",
        listener.local_addr(),
        task::current().id()
    );
}

async fn create_peer(stream: TcpStream, sender: ControlSender, broker: &Arc<Broker>) {
    match stream.peer_addr() {
        Err(e) => error!("Cannot get peer addr: {:?}", e),
        Ok(peer_addr) => {
            info!("Incoming connection from {}", peer_addr);
            // New peer (without client for now)
            // Create the packet send/receive channel
            // Launch the sender loop
            // Create peer
            // Launch the listen peer loop
            let stream = Arc::new(stream);

            let (packet_sender, packet_receiver) = mpsc::unbounded();
            let sender_handle = task::spawn(service::send_loop(packet_receiver, stream.clone()));
            let peer = Peer::new(packet_sender, sender_handle);
            let peer = Arc::new(RwLock::new(peer));

            task::spawn(service::listen_peer(
                peer,
                sender,
                broker.settings.read().await.keep_alive,
                stream,
            ));
        }
    }
}
