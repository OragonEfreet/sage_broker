use crate::Session;
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;

/// Pouet
#[async_trait]
pub trait SessionsBackEnd {
    /// pif
    async fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>>;
    ///paf
    async fn add(&mut self, session: Arc<RwLock<Session>>);
}
