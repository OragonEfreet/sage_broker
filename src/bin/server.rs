use sage_broker::{service, Broker};

#[async_std::main]
async fn main() {
    pretty_env_logger::init();

    if let Some(listener) = service::bind("localhost:6788").await {
        let config = Broker {
            keep_alive: 5,
            ..Default::default()
        };
        service::start(listener, config).await;
    }
}
