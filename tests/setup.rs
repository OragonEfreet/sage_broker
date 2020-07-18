use async_std::{
    net::TcpStream,
    task::{self, JoinHandle},
};
use futures::channel::mpsc;
use sage_broker::{service, BrokerConfig};
use std::{thread, time::Duration};

pub fn prepare_connection(config: BrokerConfig) -> (JoinHandle<()>, TcpStream) {
    let (event_sender, event_receiver) = mpsc::unbounded();

    task::spawn(service::event_loop(event_receiver));

    let handle = service::start(config.build(event_sender));

    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // panic
    for _ in 0u8..5u8 {
        if let Ok(stream) = task::block_on(TcpStream::connect("localhost:6788")) {
            return (handle, stream);
        }

        thread::sleep(Duration::from_secs(1));
    }

    panic!("Cannot establish connection");
}
