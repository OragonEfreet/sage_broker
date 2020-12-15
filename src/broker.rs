use crate::{BrokerSettings, Session};
use async_std::{
    future::Future,
    sync::{Arc, RwLock},
    task::{self, JoinHandle},
};
use futures::future;

/// The broker instance. Holds and the data related to broker configuration and
/// clients management.
pub struct Broker {
    /// The broker configuration
    pub settings: RwLock<BrokerSettings>,
    shutdown: RwLock<bool>,
    pub(crate) clients: RwLock<Vec<Arc<Session>>>,
    join_handles: RwLock<Vec<JoinHandle<()>>>,
}

impl Broker {
    /// Creates a new Arc<Broker> instance by stating its configuration.
    /// The instance is not started yet and must be given to `start`.
    pub fn build(settings: BrokerSettings) -> Arc<Self> {
        Broker {
            settings: settings.into(),
            shutdown: false.into(),
            clients: Default::default(),
            join_handles: Default::default(),
        }
        .into()
    }

    /// Asks for safe shutdown of the broker.
    /// The broker won't be stopped immediately, as running tasks will end
    /// as soon as they can do it safely.
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
    }

    /// Checks whether the broker is in shutdown state.
    /// If this value is `true`, it cannot be `false` again.
    pub async fn is_shutting_down(&self) -> bool {
        *self.shutdown.read().await
    }

    /// Spawns a task and register the associated JoinHandle into
    /// an internal collection for further retrieval
    pub async fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.join_handles.write().await.push(task::spawn(future));
    }

    /// Wait for all registered tasks (with spawn) to end.
    /// This function does not ask the tasks to be stopped. It must be done
    /// elsewhere.
    pub async fn wait_pending(&self) {
        future::join_all(self.join_handles.write().await.iter_mut()).await;

        //        for handle in self.join_handles.write().await.iter_mut() {
        //            handle.await;
        //        }
    }
}
