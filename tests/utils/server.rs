use async_std::{
    channel,
    net::{SocketAddr, TcpListener},
    sync::Arc,
    task::{self, JoinHandle},
};
use sage_broker::{service, Broker, BrokerSettings, CommandReceiver, Trigger};

pub async fn spawn(
    settings: BrokerSettings,
) -> (
    Arc<Broker>,
    JoinHandle<CommandReceiver>,
    SocketAddr,
    Trigger,
) {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    let shutdown = Trigger::default();

    let broker = Arc::new(Broker {
        settings: Arc::new(settings),
        ..Default::default()
    });

    let service_task = task::spawn(run_server(listener, broker.clone(), shutdown.clone()));

    (broker, service_task, local_addr, shutdown)
}

pub async fn stop(trigger: Trigger, service: JoinHandle<CommandReceiver>) -> CommandReceiver {
    trigger.fire().await;
    service.await
}

async fn run_server(
    listener: TcpListener,
    broker: Arc<Broker>,
    shutdown: Trigger,
) -> CommandReceiver {
    let (command_sender, command_receiver) = channel::unbounded();
    let command_loop = task::spawn(service::command_loop(
        broker.clone(),
        command_receiver,
        shutdown.clone(),
    ));
    service::listen_tcp(listener, command_sender, broker, shutdown).await;
    command_loop.await
}
