use crate::{
    service::{self, Event, EventReceiver, EventSender},
    Broker,
};
use async_std::prelude::*;

use log::info;
use std::sync::Arc;

pub async fn event_loop(
    config: Arc<Broker>,
    event_sender: EventSender,
    mut event_receiver: EventReceiver,
) {
    loop {
        while let Some(event) = event_receiver.next().await {
            match event {
                Event::NewPeer(stream) => {
                    // Start the connection loop for this stream
                    service::listen_peer(config.timeout_delay, event_sender.clone(), stream);
                }
                Event::Control(_) => {
                    info!("We got a packet!");
                }
            }
        }
    }
}
