use async_std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
    task::{self, JoinHandle},
};
use futures::channel::mpsc;
use sage_broker::{service, BrokerSettings, CommandReceiver, Sessions, Trigger};

pub async fn spawn(
    settings: BrokerSettings,
) -> (
    Arc<BrokerSettings>,
    Sessions,
    JoinHandle<CommandReceiver>,
    SocketAddr,
    Trigger,
) {
    let listener = TcpListener::bind("localhost:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    let settings = Arc::new(settings);

    let shutdown = Trigger::default();

    let sessions = Sessions::default();
    let service_task = task::spawn(run_server(
        listener,
        sessions.clone(),
        settings.clone(),
        shutdown.clone(),
    ));

    (settings, sessions, service_task, local_addr, shutdown)
}

pub async fn stop(trigger: Trigger, service: JoinHandle<CommandReceiver>) -> CommandReceiver {
    trigger.fire().await;
    service.await
}

async fn run_server(
    listener: TcpListener,
    sessions: Sessions,
    settings: Arc<BrokerSettings>,
    shutdown: Trigger,
) -> CommandReceiver {
    let (command_sender, command_receiver) = mpsc::unbounded();
    let command_loop = task::spawn(service::command_loop(
        sessions,
        settings.clone(),
        command_receiver,
        shutdown.clone(),
    ));
    service::listen_tcp(listener, command_sender, settings, shutdown).await;
    command_loop.await
}
