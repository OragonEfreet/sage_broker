use async_std::task;
use futures::channel::mpsc;
use sage_broker::{service, BrokerConfig};

fn main() {
    pretty_env_logger::init();

    let (event_sender, event_receiver) = mpsc::unbounded();
    let event_loop = task::spawn(service::event_loop(event_receiver));

    let server = service::start(BrokerConfig::new("localhost:6788").build(event_sender));
    task::block_on(server);
    task::block_on(event_loop);
}
