use async_std::task;
use sage_broker::{service, BrokerConfig};

fn main() {
    pretty_env_logger::init();
    let server = service::start(BrokerConfig::new("localhost:6788").build());
    task::block_on(server);
}
