use async_std::{
    channel,
    net::{SocketAddr, TcpListener},
    sync::{Arc, RwLock},
    task::{self, JoinHandle},
};
use sage_broker::{service, BrokerSettings, CommandReceiver, Sessions, Subscriptions, Trigger};

pub async fn spawn(
    settings: BrokerSettings,
) -> (
    Arc<RwLock<Sessions>>,
    Arc<RwLock<Subscriptions>>,
    JoinHandle<CommandReceiver>,
    SocketAddr,
    Trigger,
) {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    let shutdown = Trigger::default();
    let sessions = Arc::new(RwLock::new(Sessions::default()));
    let subscriptions = Arc::new(RwLock::new(Subscriptions::default()));
    let settings = Arc::new(settings);

    let service_task = task::spawn(run_server(
        listener,
        settings.clone(),
        sessions.clone(),
        subscriptions.clone(),
        shutdown.clone(),
    ));

    (sessions, subscriptions, service_task, local_addr, shutdown)
}

pub async fn stop(trigger: Trigger, service: JoinHandle<CommandReceiver>) -> CommandReceiver {
    trigger.fire().await;
    service.await
}

async fn run_server(
    listener: TcpListener,
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    subscriptions: Arc<RwLock<Subscriptions>>,
    shutdown: Trigger,
) -> CommandReceiver {
    let (command_sender, command_receiver) = channel::unbounded();
    let command_loop = task::spawn(service::command_loop(
        settings.clone(),
        sessions,
        subscriptions,
        command_receiver,
        shutdown.clone(),
    ));
    service::listen_tcp(listener, command_sender, settings, shutdown).await;
    command_loop.await
}
