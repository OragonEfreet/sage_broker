use sage_broker::{service, BrokerSettings, CommandReceiver, Sessions, Trigger};
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::{
    net::TcpListener,
    sync::mpsc,
    task::{self, JoinHandle},
};

pub async fn spawn(
    settings: BrokerSettings,
) -> (
    Arc<RwLock<Sessions>>,
    JoinHandle<CommandReceiver>,
    SocketAddr,
    Trigger,
) {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    let shutdown = Trigger::default();
    let sessions = Arc::new(RwLock::new(Sessions::default()));
    let settings = Arc::new(settings);

    let service_task = task::spawn(run_server(
        listener,
        settings.clone(),
        sessions.clone(),
        shutdown.clone(),
    ));

    (sessions, service_task, local_addr, shutdown)
}

pub async fn stop(trigger: Trigger, service: JoinHandle<CommandReceiver>) -> CommandReceiver {
    trigger.fire();
    service.await.unwrap()
}

async fn run_server(
    listener: TcpListener,
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    shutdown: Trigger,
) -> CommandReceiver {
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let command_loop = task::spawn(service::command_loop(
        settings.clone(),
        sessions,
        command_receiver,
        shutdown.clone(),
    ));
    service::listen_tcp(listener, command_sender, settings, shutdown).await;
    command_loop.await.unwrap()
}
