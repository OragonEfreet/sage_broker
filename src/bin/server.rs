use async_std::net::{TcpListener, ToSocketAddrs};
use async_std::{channel, sync::Arc, task};
use log::{error, info};
use sage_broker::{service, BrokerSettings, Trigger};

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    if let Some(listener) = bind("localhost:1883").await {
        let settings = Arc::new(BrokerSettings {
            keep_alive: 0,
            ..BrokerSettings::default()
        });

        let shutdown = Trigger::default();

        // Create the command packet channel and spawn a new
        // task for command packet treatment.
        let (command_sender, command_receiver) = channel::unbounded();

        info!("Creating the command loop...");
        let command_loop = task::spawn(service::command_loop(
            Default::default(), // The list of sessions
            settings.clone(),   // The settings
            command_receiver,   // The command receiver to use
            shutdown.clone(),   // Shutdown trigger
        ));

        // Launch the listen server.
        // This is the main task, responsible for listening the Tcp connexions
        // And creating new peers from it.
        info!("Creating the listen loop...");
        let server = task::spawn(service::listen_tcp(
            listener,
            command_sender,
            settings,
            shutdown.clone(),
        ));

        // Use the ctrlc crate to handle manual termination
        ctrlc::set_handler(move || {
            if task::block_on(shutdown.is_fired()) {
                std::process::exit(0);
            } else {
                task::block_on(shutdown.fire())
            }
        })
        .expect("Error setting Ctrl-C handler");

        server.await;
        info!("Listen loop ended");

        // When `broker.is_shutting_down().await` returns true, `listen_tcp` will
        // complete by itself.

        // -----------------------------------------------------------------------
        // Graceful close
        // A lot of tasks have been spawned during the server lifetime.
        // To gracefully stop all of them, follow these instructions:
        //
        // First, graceful close is made by waiting `listen_tcp` to complete.
        // In our case we awaited it. So we're sure about it.
        // If `listen_tcp` still runs, it may create new connexions, hence the
        // importance of waiting for it.

        // At this point peers won't process any incoming data and thus no
        // new command will be generated.
        // All peers being flagged as closed, their respective `listen_peer` tasks
        // will all end.
        // `listen_peer` hold a `CommandSender` instance, which means that once
        // they are all complete, `command_loop` will end.
        // Thus, by awaiting for `command_loop`, we ensure all `listen_peer` and
        // `command_loop` are done.
        //
        // Command loop will itself wait for the end of all pending tasks
        // registered by any other part of the server
        info!("Waiting for command loop to complete...");
        command_loop.await;

        info!("Done.");
    }
}

/// Utility function that opens a Tcp connection for listening, returning some
/// `TcpListener` in case of success, `None` otherwise.
/// The function does not perform anything special apart from opening the
/// connexion, meaning you can provide your own instance of `TcpListener` to
/// `listen`.
pub async fn bind(addr: &str) -> Option<TcpListener> {
    let addr = String::from(addr);

    if let Ok(addrs) = addr.to_socket_addrs().await {
        let addrs = addrs
            .map(|addr| addr.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        if let Ok(listener) = TcpListener::bind(addr).await {
            info!("Tcp bound to {}", listener.local_addr().unwrap());
            Some(listener)
        } else {
            error!("Cannot listen from {}", addrs);
            None
        }
    } else {
        error!("Cannot compute addresses from {}", addr);
        None
    }
}
