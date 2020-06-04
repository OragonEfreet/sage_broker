use async_std::task;
use log::error;
use pretty_env_logger;
use sage_broker::BrokerBuilder;

fn main() {
    pretty_env_logger::init();

    let broker = BrokerBuilder::default().start();

    if let Err(e) = task::block_on(broker.listen("localhost:6788")) {
        error!("{}", e.to_string());
    }
}
