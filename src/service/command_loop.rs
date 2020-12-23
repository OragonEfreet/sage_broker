use crate::{
    treat::{treat, TreatAction},
    BrokerSettings, Command, CommandReceiver, Peer, Sessions, Trigger,
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
/// Eventually, this task may be a spawner for other tasks
pub async fn command_loop(
    mut sessions: Sessions,
    settings: Arc<BrokerSettings>,
    mut from_command_channel: CommandReceiver,
    shutdown: Trigger,
) -> (Sessions, CommandReceiver) {
    info!("Start command loop");
    while let Some(command) = from_command_channel.next().await {
        // Currently can only be Command::Control

        let Command::Control(peer, packet) = command;
        control_packet(&settings, &mut sessions, packet, peer, &shutdown).await;
    }
    info!("Stop command loop");

    (sessions, from_command_channel)
}

async fn control_packet(
    settings: &Arc<BrokerSettings>,
    sessions: &mut Sessions,
    packet: Packet,
    source: Arc<RwLock<Peer>>,
    shutdown: &Trigger,
) {
    match treat(&settings, sessions, packet, &source, shutdown).await {
        TreatAction::Respond(packet) => source.write().await.send(packet).await,
        TreatAction::RespondAndDisconnect(packet) => source.write().await.send_close(packet).await,
    };
}