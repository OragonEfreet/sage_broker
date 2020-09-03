use sage_broker::{service, BrokerSettings};

#[async_std::main]
async fn main() {
    pretty_env_logger::init();

    if let Some(listener) = service::bind("localhost:6788").await {
        let settings = BrokerSettings {
            keep_alive: 5,
            ..Default::default()
        };
        service::start(listener, settings.into()).await;
    }
}
