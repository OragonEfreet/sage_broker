use crate::Sessions;
use async_std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
/// Holds the data manipulated by the broker
#[derive(Clone, Default)]
pub struct BackEnd {
    sessions: Arc<RwLock<Sessions>>,
}

impl BackEnd {
    /// Returns a refrence to the sessions
    pub async fn sessions(&self) -> RwLockReadGuard<'_, Sessions> {
        self.sessions.read().await
    }

    /// Returns a mutable reference to the sessions
    pub async fn sessions_mut(&self) -> RwLockWriteGuard<'_, Sessions> {
        self.sessions.write().await
    }
}

pub struct SubscriptionView;
