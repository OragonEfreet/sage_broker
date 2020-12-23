use crate::Session;
use async_std::sync::{Arc, RwLock};

/// Pouet
pub trait SessionsBackEnd {
    /// pif
    fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>>;
    ///paf
    fn add(&mut self, session: Arc<RwLock<Session>>);
}
