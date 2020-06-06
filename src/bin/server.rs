use async_std::task;
use sage_broker::Broker;

fn main() {
    pretty_env_logger::init();
    let server = Broker::new("localhost:6788").run();

    task::block_on(server);
}
