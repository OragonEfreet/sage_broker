use async_std::{
    channel,
    net::{SocketAddr, TcpListener},
    sync::{Arc, RwLock},
    task::{self, JoinHandle},
};
use sage_broker::{service, Broker, BrokerSettings, CommandReceiver, Sessions, Trigger};

pub async fn spawn(
    settings: BrokerSettings,
) -> (
    Arc<RwLock<Sessions>>,
    Arc<Broker>,
    JoinHandle<CommandReceiver>,
    SocketAddr,
    Trigger,
) {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    let shutdown = Trigger::default();
    let sessions = Arc::new(RwLock::new(Sessions::default()));
    let settings = Arc::new(settings);
    let broker = Arc::new(Broker::default());

    let service_task = task::spawn(run_server(
        listener,
        settings.clone(),
        sessions.clone(),
        broker.clone(),
        shutdown.clone(),
    ));

    (sessions, broker, service_task, local_addr, shutdown)
}

pub async fn stop(trigger: Trigger, service: JoinHandle<CommandReceiver>) -> CommandReceiver {
    trigger.fire().await;
    service.await
}

async fn run_server(
    listener: TcpListener,
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    broker: Arc<Broker>,
    shutdown: Trigger,
) -> CommandReceiver {
    let (command_sender, command_receiver) = channel::unbounded();
    let command_loop = task::spawn(service::command_loop(
        settings.clone(),
        sessions,
        broker.clone(),
        command_receiver,
        shutdown.clone(),
    ));
    service::listen_tcp(listener, command_sender, settings, shutdown).await;
    command_loop.await
}
