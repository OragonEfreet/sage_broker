use async_std::task;
use sage_broker::Broker;

fn main() {
    pretty_env_logger::init();
    task::block_on(Broker::new("localhost:6788").run());
}
