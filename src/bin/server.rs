use async_std::task;
use sage_broker::service;

fn main() {
    pretty_env_logger::init();
    let server = service::start(Default::default());
    task::block_on(server);
}
