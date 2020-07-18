use async_std::task;
use sage_broker::{service, Broker, BrokerConfig};

fn main() {
    pretty_env_logger::init();
    let server = service::start(Broker::from_config(BrokerConfig::new("localhost:6788")));
    task::block_on(server);
}
