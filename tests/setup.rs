use async_std::{
    net::TcpStream,
    task::{self, JoinHandle},
};
use sage_broker::{service, Broker};
use std::{thread, time::Duration};

pub fn prepare_connection(config: Broker) -> (JoinHandle<()>, TcpStream) {
    let handle = service::start(config);

    // Makes 5 connexion attemps, every 1 second until a connexion is made, or
    // pannic
    for _ in 0u8..5u8 {
        if let Ok(stream) = task::block_on(TcpStream::connect("localhost:6788")) {
            return (handle, stream);
        }

        thread::sleep(Duration::from_secs(1));
    }

    panic!("Cannot establish connection");
}
