use crate::{
    service::{self, sender_loop},
    Event, EventReceiver, Peer,
};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Packet, ReasonCode};

pub async fn event_loop(mut event_receiver: EventReceiver) {
    info!("Start event loop ({})", task::current().id());
    while let Some(event) = event_receiver.next().await {
        debug!("Event ({}): {}", task::current().id(), event);
        match event {
            // Ending a peer
            Event::EndPeer(_) => {
                // info!("End peer {}", peer.read().await.id());
            }
            // Creating a new peer
            Event::NewPeer(broker, stream) => {
                match stream.peer_addr() {
                    Err(e) => error!("Cannot get peer addr: {:?}", e),
                    Ok(_) => {
                        // New peer (no client for now)
                        // Create the packet send/receive channel
                        // Launch the sender loop
                        // Create peer
                        // Launch the listen peer loop
                        let stream = Arc::new(stream);

                        let (packet_sender, packet_receiver) = mpsc::unbounded();
                        let sender_handle =
                            task::spawn(sender_loop(packet_receiver, stream.clone()));
                        let peer = Peer::new(packet_sender, sender_handle);
                        let peer = Arc::new(RwLock::new(peer));

                        // Start the connection loop for this stream
                        service::listen_peer(peer, broker, stream);
                    }
                }
            }
            // Dispatch to the corresponding function
            Event::Control(broker, peer, packet) => match packet {
                Packet::Connect(packet) => {
                    broker.connect(peer, packet).await;
                }
                _ => {
                    error!("Unsupported packet");
                    let packet = ConnAck {
                        reason_code: ReasonCode::ImplementationSpecificError,
                        ..Default::default()
                    }
                    .into();
                    peer.write().await.close(Some(packet)).await;
                }
            },
        }
    }
    info!("Stop event loop {}", task::current().id());
}
