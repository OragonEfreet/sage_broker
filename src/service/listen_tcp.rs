use crate::{service, BrokerSettings, CommandSender, Peer, Trigger};
use futures::future::join_all;
use log::{error, info};
use std::{sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
    task::{self, JoinHandle},
    time,
};

/// Creates a channel for control packets and starts the command loop and the
/// listen Tcp loop.
/// `listener` can be any instance of `async_std::net::TcpListener` but you can
/// use `bind` to obtain one.
/// Task ends when the shutdown object is triggered.
/// It owns all peer listen/send in order to wait for them once the server is stopping.
pub async fn listen_tcp(
    listener: TcpListener,
    to_command_channel: CommandSender,
    settings: Arc<BrokerSettings>,
    shutdown: Trigger,
) {
    // Listen to any connection
    info!(
        "Start listening from '{:?}'",
        listener.local_addr().unwrap(),
    );

    let listen_timeout = Duration::from_secs(1);

    let mut tcp_listeners = Vec::new();
    let mut tcp_senders = Vec::new();

    while !shutdown.is_fired() {
        // Listen for 1 second for an incoming connexion
        if let Ok(result) = time::timeout(listen_timeout, listener.accept()).await {
            match result {
                Err(e) => error!("Cannot accept Tcp stream: {}", e.to_string()),
                Ok((stream, _)) => {
                    if let Some((listener, sender)) = create_peer(
                        stream,
                        to_command_channel.clone(),
                        settings.clone(),
                        shutdown.clone(),
                    )
                    .await
                    {
                        tcp_listeners.push(listener);
                        tcp_senders.push(sender);
                    }
                }
            }
        }
    }
    info!("Stop listening from '{:?}'", listener.local_addr().unwrap(),);

    info!("Waiting for listeners end...");
    join_all(tcp_listeners).await;
    info!("Waiting for senders end...");
    join_all(tcp_senders).await;
}

async fn create_peer(
    stream: TcpStream,
    command_sender: CommandSender,
    settings: Arc<BrokerSettings>,
    shutdown: Trigger,
) -> Option<(JoinHandle<()>, JoinHandle<()>)> {
    match stream.peer_addr() {
        Err(e) => {
            error!("Cannot get peer addr: {:?}", e);
            None
        }
        Ok(peer_addr) => {
            info!("Incoming connection from '{}'", peer_addr);

            // New peer
            // Create the packet send/receive channel
            // Launch the packet sender loop
            // Create peer
            // Launch the listen peer loop

            let (packet_sender, packet_receiver) = mpsc::unbounded_channel();

            // The send_peer task will end as long as no packet_sender is
            // open anymore.
            // The packet sender is held in the Peer instance, meaning that
            // it is alive as long as the listen_peer is, and any pending
            // task temporary keeping the Peer alive (Command Packets)
            let (rd, wr) = stream.into_split();
            let sender_task = task::spawn(service::send_peer(packet_receiver, wr));

            // No need to handle this one, a safe close
            // Will always terminate it before the command_loop
            // See "service::run" for example
            let listen_task = task::spawn(service::listen_peer(
                Peer::new(peer_addr, packet_sender),
                command_sender,
                settings.keep_alive,
                rd,
                shutdown,
            ));

            Some((listen_task, sender_task))
        }
    }
}
