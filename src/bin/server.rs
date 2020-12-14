use async_std::task;
use sage_broker::{service, Broker};

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    if let Some(listener) = service::bind("localhost:1883").await {
        let broker = Broker::build(Default::default());
        let service = task::spawn(service::run(listener, broker.clone()));

        //use std::time::Duration;
        //task::sleep(Duration::from_secs(5)).await;
        //println!("Shutdown");
        //broker.shutdown().await;

        service.await;
    }
}
