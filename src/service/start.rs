use crate::{service, Broker};
use async_std::{
    net::TcpListener,
    sync::{Arc, RwLock},
    task,
};
use futures::channel::mpsc;

/// Utility function which creates a channel for control packets and starts
/// the control loop and the listen Tcp loop.
/// `listener` can be any instance of `async_std::net::TcpListener` but you can
/// use `bind` to obtain one.
pub async fn start(listener: TcpListener, config: Broker) {
    let (control_sender, control_receiver) = mpsc::unbounded();
    let config = Arc::new(RwLock::new(config));

    // TODO This task should be waited in listen_tcp
    task::spawn(service::control_loop(config.clone(), control_receiver));

    service::listen_tcp(listener, control_sender, config).await;
}
