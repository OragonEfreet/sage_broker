use crate::{
    service::{self, send_loop},
    Broker, Event, EventReceiver, EventSender, Peer,
};
use async_std::{
    net::TcpStream,
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use log::{debug, error, info};
use sage_mqtt::{ConnAck, Packet, ReasonCode};

// An event loop is made for each started broker
// The event_loop is responsible for maintaining most data related to
// the broker, such as the list of clients.
pub async fn event_loop(
    config: Arc<Broker>,
    event_sender: EventSender,
    mut event_receiver: EventReceiver,
) {
    info!("Start event loop ({})", task::current().id());
    while let Some(event) = event_receiver.next().await {
        debug!("Event ({}): {}", task::current().id(), event);
        match event {
            Event::EndPeer(_) => debug!("End peer"),
            Event::NewPeer(stream) => create_peer(event_sender.clone(), &config, stream).await,
            Event::Control(peer, packet) => treat_packet(peer, packet).await,
        }
    }
    info!("Stop event loop {}", task::current().id());
}

// Upon receiving `packet` from a given `peer`, the function must dispatch to
// the corresponding function according to the packet type.
async fn treat_packet(peer: Arc<RwLock<Peer>>, packet: Packet) {
    match packet {
        Packet::Connect(_) => {
            // broker.connect(peer, packet).await;
            // TODO
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
    };
}

// Creation of a new peer involves a new `Peer` instance along with starting
// an async loops for receiving (`listen_loop`) and sending (`sending_loop`)
// packets.
async fn create_peer(event_sender: EventSender, config: &Broker, stream: TcpStream) {
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
            let sender_handle = task::spawn(send_loop(packet_receiver, stream.clone()));
            let peer = Peer::new(packet_sender, sender_handle);
            let peer = Arc::new(RwLock::new(peer));

            // Start the connection loop for this stream
            task::spawn(service::listen_loop(
                peer,
                event_sender,
                config.timeout_delay,
                stream,
            ));
        }
    };
}
