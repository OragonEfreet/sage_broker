use crate::{service, Broker, ControlSender, Peer};
use async_std::{
    future,
    net::{TcpListener, TcpStream},
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use futures::SinkExt;
use log::{error, info};
use std::time::Duration;

/// Creates a channel for control packets and starts the control loop and the
/// listen Tcp loop.
/// `listener` can be any instance of `async_std::net::TcpListener` but you can
/// use `bind` to obtain one.
pub async fn listen_tcp(
    listener: TcpListener,
    to_control_channel: ControlSender,
    broker: Arc<Broker>,
) {
    // Listen to any connection
    info!(
        "Start listening from '{:?}'",
        listener.local_addr().unwrap(),
    );

    let listen_timeout = Duration::from_secs(1);

    while !broker.is_shutting_down().await {
        // Listen for 1 second for an incoming connexion
        if let Ok(result) = future::timeout(listen_timeout, listener.accept()).await {
            match result {
                Err(e) => error!("Cannot accept Tcp stream: {}", e.to_string()),
                Ok((stream, _)) => create_peer(stream, to_control_channel.clone(), &broker).await,
            }
        }
    }
    info!("Stop listening from '{:?}'", listener.local_addr().unwrap(),);
}

async fn create_peer(stream: TcpStream, mut control_sender: ControlSender, broker: &Arc<Broker>) {
    match stream.peer_addr() {
        Err(e) => error!("Cannot get peer addr: {:?}", e),
        Ok(peer_addr) => {
            info!("Incoming connection from '{}'", peer_addr);

            // New peer
            // Create the packet send/receive channel
            // Launch the packet sender loop
            // Create peer
            // Launch the listen peer loop
            let stream = Arc::new(stream);

            let (packet_sender, packet_receiver) = mpsc::unbounded();

            // This is equivalent to task::spawn but the handle will be
            // kept into an internal collection for further await-al
            let task = task::spawn(service::send_peer(packet_receiver, stream.clone()));

            if let Err(e) = control_sender.send(task.into()).await {
                error!("Cannot send task control: {:?}", e);
            }

            let peer = Peer::new(peer_addr, packet_sender);
            let peer = Arc::new(RwLock::new(peer));

            // No need to handle this one, a safe close
            // Will always terminate it before the control_loop
            // See "service::run" for example
            task::spawn(service::listen_peer(
                peer,
                control_sender,
                broker.clone(),
                stream,
            ));
        }
    }
}
