use async_std::task;
use futures::channel::mpsc;
use log::info;
use sage_broker::{service, Broker, BrokerSettings};

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    if let Some(listener) = service::bind("localhost:1883").await {
        let broker = Broker::build(BrokerSettings {
            keep_alive: 0,
            ..Default::default()
        });

        //let service = task::spawn(service::run(listener, broker.clone()));
        //service.await;

        // Create the control packet channel and spawn a new
        // task for control packet treatment.
        let (control_sender, control_receiver) = mpsc::unbounded();
        info!("Creating the control loop...");
        let control_loop = task::spawn(service::control_loop(broker.clone(), control_receiver));

        // Launch the listen server.
        // This is the main task, responsible for listening the Tcp connexions
        // And creating new peers from it.
        // In this example, we await it directly, but you may launch it
        // and join it later.
        info!("Creating the listen loop...");
        service::listen_tcp(listener, control_sender, broker.clone()).await;
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
        // new Control will be generated.
        // All peers being flagged as closed, their respective `listen_peer` tasks
        // will all end.
        // `listen_peer` hold a `ControlSender` instance, which means that once
        // they are all complete, `control_loop` will end.
        // Thus, by awaiting for `control_loop`, we ensure all `listen_peer` and
        // `control_loop` are done.
        info!("Waiting for control loop to complete...");
        control_loop.await;

        // At this point the server won't receive nor process anything, but pending
        // process may still be running:
        // - Packets to be sent
        // All pending operations can be awaited with:
        info!("Waiting for other tasks to complete...");
        broker.wait_pending().await;

        info!("Done.");
    }
}
