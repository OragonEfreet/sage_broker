use crate::{control, BrokerSettings, CommandReceiver, Sessions, Trigger};
use async_std::{
    prelude::*,
    sync::{Arc, RwLock},
};
use log::{debug, info};
use sage_mqtt::{Disconnect, ReasonCode};

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
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    mut from_command_channel: CommandReceiver,
    shutdown: Trigger,
) -> CommandReceiver {
    info!("Start command loop");
    while let Some((peer, packet)) = from_command_channel.next().await {
        debug!(
            "[{:?}] <<< {:?}",
            if let Some(s) = peer.read().await.session().await {
                s.read().await.client_id().into()
            } else {
                String::from("N/A")
            },
            packet
        );
        // If the broker is stopping, let's notify here the client with a
        // DISCONNECT and close the peer
        if shutdown.is_fired().await {
            peer.write()
                .await
                .send_close(
                    Disconnect {
                        reason_code: ReasonCode::ServerShuttingDown,
                        ..Default::default()
                    }
                    .into(),
                )
                .await;
        } else {
            control::packet(packet, sessions.clone(), settings.clone(), &peer).await;
        };
    }
    info!("Stop command loop");
    from_command_channel
}
