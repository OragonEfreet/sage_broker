use crate::{service, Broker};
use async_std::{
    net::TcpListener,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;

pub async fn listen(listener: TcpListener, config: Broker) {
    let (event_sender, event_receiver) = mpsc::unbounded();
    let config = Arc::new(RwLock::new(config));

    // TODO This task should be waited in listen_tcp
    task::spawn(service::event_loop(config.clone(), event_receiver));

    service::listen_tcp(listener, event_sender).await;
}
