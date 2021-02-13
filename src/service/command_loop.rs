use crate::{Action, BackEnd, BrokerSettings, CommandReceiver, Control, Peer, Trigger};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
};
use log::{debug, info};
use sage_mqtt::{Disconnect, Packet, ReasonCode};

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
    backend: BackEnd,
    settings: Arc<BrokerSettings>,
    mut from_command_channel: CommandReceiver,
    shutdown: Trigger,
) -> CommandReceiver {
    info!("Start command loop");
    while let Some(command) = from_command_channel.next().await {
        // Currently can only be Command::Control

        let (peer, packet) = command;
        control_packet(&settings, &backend, packet, peer, &shutdown).await;
    }
    info!("Stop command loop");

    from_command_channel
}

async fn control_packet(
    settings: &Arc<BrokerSettings>,
    backend: &BackEnd,
    packet: Packet,
    source: Arc<RwLock<Peer>>,
    shutdown: &Trigger,
) {
    debug!(
        "[{:?}] <<< {:?}",
        if let Some(s) = source.read().await.session() {
            s.read().await.client_id().into()
        } else {
            String::from("N/A")
        },
        packet
    );
    // If the broker is stopping, let's notify here the client with a
    // DISCONNECT and close the peer
    // NOTE Maybe move this test up a bit
    let action = if shutdown.is_fired().await {
        Action::RespondAndDisconnect(
            Disconnect {
                reason_code: ReasonCode::ServerShuttingDown,
                ..Default::default()
            }
            .into(),
        )
    } else {
        packet.control(backend, &settings, &source).await
    };
    match action {
        Action::Respond(packet) => source.write().await.send(packet).await,
        Action::RespondAndDisconnect(packet) => source.write().await.send_close(packet).await,
    };
}
