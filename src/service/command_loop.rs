use crate::{control, BrokerSettings, CommandReceiver, Publisher, Sessions, Trigger};
use log::{debug, error, info};
use sage_mqtt::{Disconnect, ReasonCode};
use std::sync::{Arc, RwLock};

/// The command loop is reponsible from receiving and treating any command
/// packet. It thus represents the actual instance of a running broker.
/// The loop holds and manages the list of sessions, dispatching messages from
/// client to client.
/// The loop automatically ends when all command sender channels are dropped.
/// These are held by `listen_loop` (one per peer) and the `listen_tcp`
/// tasks. Meaning when all peers are dropped and port listenning is stopped
/// The command loop ends.
/// Eventually, this task may become a spawner for other tasks
pub async fn command_loop(
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    mut from_command_channel: CommandReceiver,
    shutdown: Trigger,
) -> CommandReceiver {
    let publisher = Arc::new(Publisher::default());
    // Validate broker settings against current limitations
    if !settings.is_valid() {
        error!("Shutting down server due to current limitations");
        shutdown.fire();
    }

    info!("Start command loop");
    while let Some((peer, packet)) = from_command_channel.recv().await {
        debug!(
            "[{:?}] <<< {:#?}",
            if let Some(s) = peer.session() {
                s.client_id().into()
            } else {
                String::from("N/A")
            },
            packet
        );
        // If the broker is stopping, let's notify here the client with a
        // DISCONNECT and close the peer
        if shutdown.is_fired() {
            peer.send_close(
                Disconnect {
                    reason_code: ReasonCode::ServerShuttingDown,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            control::run(
                settings.clone(),
                sessions.clone(),
                packet,
                peer,
                publisher.clone(),
            )
            .await;
        };
    }
    info!("Stop command loop");
    from_command_channel
}
