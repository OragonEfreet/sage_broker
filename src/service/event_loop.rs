use crate::{
    service::{self, sender_loop},
    BrokerConfig, Event, EventReceiver, EventSender, Peer, PeerState,
};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;
use log::{error, info};
use sage_mqtt::{ConnAck, Connect, Packet, ReasonCode};

async fn process_connect(_: Connect, peer: Arc<RwLock<Peer>>) {
    let out = ConnAck::default();
    let mut peer = peer.write().await;
    peer.set_state(PeerState::Active);

    // let with_prob_info = packet.request_problem_information;

    // if packet.authentication.is_some() {
    //     out.reason_code = ReasonCode::BadAuthenticationMethod;
    //     if with_prob_info {
    //         out.reason_string = "Enhanced authentication not available".into();
    //     }
    // }

    peer.send(out.into()).await;
}

pub async fn event_loop(
    config: Arc<BrokerConfig>,
    event_sender: EventSender,
    mut event_receiver: EventReceiver,
) {
    while let Some(event) = event_receiver.next().await {
        match event {
            // Ending a peer
            Event::EndPeer(peer) => {
                info!("End peer {}", peer.read().await.id());
            }
            // Creating a new peer
            Event::NewPeer(stream) => {
                match stream.peer_addr() {
                    Err(e) => error!("Cannot get peer addr: {:?}", e),
                    Ok(_) => {
                        // New peer
                        // Create the packet send/receive channel
                        // Launch the sender loop
                        // Create peer
                        // Launch the listen peer loop
                        let stream = Arc::new(stream);

                        let (packet_sender, packet_receiver) = mpsc::unbounded();
                        let sender_handle =
                            task::spawn(sender_loop(packet_receiver, stream.clone()));
                        let peer = Peer::new(packet_sender, sender_handle);
                        info!("New peer: {}", peer.id());
                        let peer = Arc::new(RwLock::new(peer));

                        // Start the connection loop for this stream
                        service::listen_peer(
                            peer,
                            config.timeout_delay,
                            event_sender.clone(),
                            stream,
                        );
                    }
                }
            }
            // Dispatch to the corresponding function
            Event::Control(peer, packet) => match packet {
                Packet::Connect(packet) => {
                    process_connect(packet, peer).await;
                    // println!("Should process connect");
                }
                _ => {
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
}
