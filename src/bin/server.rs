use async_std::task;
use log::error;
use pretty_env_logger;
use sage_broker::Broker;

fn main() {
    pretty_env_logger::init();

    let broker = Broker::default().start();

    if let Err(e) = task::block_on(broker.listen("localhost:6788")) {
        error!("{}", e.to_string());
    }
}
