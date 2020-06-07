use crate::{
    service::{self, sender_loop, Event, EventReceiver, EventSender, Peer},
    Broker,
};
use async_std::{prelude::*, sync::Arc, task};
use futures::channel::mpsc;
use log::{debug, error, info};

pub async fn event_loop(
    config: Arc<Broker>,
    event_sender: EventSender,
    mut event_receiver: EventReceiver,
) {
    while let Some(event) = event_receiver.next().await {
        match event {
            Event::EndPeer(peer) => {
                debug!("End peer {:?}", peer);
            }
            Event::NewPeer(stream) => {
                match stream.peer_addr() {
                    Err(e) => error!("Cannot get peer addr: {:?}", e),
                    Ok(_) => {
                        let stream = Arc::new(stream);

                        let (packet_sender, packet_receiver) = mpsc::unbounded();
                        let sender_handle =
                            task::spawn(sender_loop(packet_receiver, stream.clone()));
                        let peer = Peer::new(packet_sender, sender_handle);

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
            Event::Control(_) => {
                info!("We got a packet!");
            }
        }
    }
}
