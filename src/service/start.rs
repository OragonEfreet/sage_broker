use crate::{service, Broker};
use async_std::{net::TcpListener, sync::Arc, task};
use futures::channel::mpsc;

/// Utility function which creates a channel for control packets and starts
/// the control loop and the listen Tcp loop.
/// `listener` can be any instance of `async_std::net::TcpListener` but you can
/// use `bind` to obtain one.
pub async fn start(listener: TcpListener, broker: Broker) {
    let (control_sender, control_receiver) = mpsc::unbounded();

    let broker = Arc::new(broker);

    // TODO This task should be waited in listen_tcp
    task::spawn(service::control_loop(broker.clone(), control_receiver));

    service::listen_tcp(listener, control_sender, broker).await;
}
