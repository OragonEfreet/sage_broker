use crate::{
    treat::{treat, TreatAction},
    Broker, Control, ControlReceiver, Peer,
};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
    task::JoinHandle,
};
use futures::future;
use log::info;
use sage_mqtt::Packet;

/// The control loop is reponsible from receiving and treating any control
/// packet. I thus represent the actual instance of a running broker.
/// The loop holds and manages the list of sessions, dispatching messages from
/// client to client.
/// The loop automatically ends when all control sender channels are dropped.
/// These are held by `listen_loop` (one per peer) and the `listen_tcp`
/// tasks. Meaning when all peers are dropped and port listenning is stopped
/// The control loop ends.
/// TODO Eventually, this task may be a spawner for other tasks
pub async fn control_loop(broker: Arc<Broker>, mut from_control_channel: ControlReceiver) {
    // Create a Vec<JoinHanle<()>> that will be used for joining all tasks
    // at the end of the control_loop
    let mut pending_tasks = Vec::new();

    info!("Start control loop");
    while let Some(control) = from_control_channel.next().await {
        match control {
            Control::Packet(peer, packet) => control_packet(&broker, packet, peer).await,
            Control::RegisterTask(task) => register_task(&mut pending_tasks, task).await,
        }
    }
    info!("Stop control loop");
    info!("Wait for {} task(s) to finish...", pending_tasks.len());
    future::join_all(pending_tasks).await;
}

async fn control_packet(broker: &Arc<Broker>, packet: Packet, source: Arc<RwLock<Peer>>) {
    match treat(&broker, packet, &source).await {
        TreatAction::Respond(packet) => source.write().await.send(packet).await,
        TreatAction::RespondAndDisconnect(packet) => source.write().await.send_close(packet).await,
    };
}

async fn register_task(tasks: &mut Vec<JoinHandle<()>>, task: JoinHandle<()>) {
    tasks.push(task);
}
