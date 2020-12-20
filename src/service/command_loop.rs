use crate::Session;
use crate::{
    treat::{treat, TreatAction},
    Broker, Command, CommandReceiver, Peer,
};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
};
use log::info;
use sage_mqtt::Packet;

/// The command loop is reponsible from receiving and treating any command
/// packet. I thus represent the actual instance of a running broker.
/// The loop holds and manages the list of sessions, dispatching messages from
/// client to client.
/// The loop automatically ends when all command sender channels are dropped.
/// These are held by `listen_loop` (one per peer) and the `listen_tcp`
/// tasks. Meaning when all peers are dropped and port listenning is stopped
/// The command loop ends.
/// TODO Eventually, this task may be a spawner for other tasks
pub async fn command_loop(broker: Arc<Broker>, mut from_command_channel: CommandReceiver) {
    // The sessions list, maintaining all active (and inactive?) sessions
    // of the broker.
    // ATM, it does not need to be RwLocked, because only this task accesses it.
    let mut sessions = Vec::new();

    info!("Start command loop");
    while let Some(command) = from_command_channel.next().await {
        // Currently can only be Command::Control

        let Command::Control(peer, packet) = command;
        control_packet(&broker, &mut sessions, packet, peer).await;
    }
    info!("Stop command loop");
}

async fn control_packet(
    broker: &Arc<Broker>,
    sessions: &mut Vec<Arc<Session>>,
    packet: Packet,
    source: Arc<RwLock<Peer>>,
) {
    match treat(&broker, sessions, packet, &source).await {
        TreatAction::Respond(packet) => source.write().await.send(packet).await,
        TreatAction::RespondAndDisconnect(packet) => source.write().await.send_close(packet).await,
    };
}
