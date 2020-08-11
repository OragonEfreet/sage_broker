use async_std::task;
use sage_broker::{service, Broker};

fn main() {
    pretty_env_logger::init();
    let server = service::start(Broker {
        keep_alive: 5,
        ..Default::default()
    });
    task::block_on(server);
}
