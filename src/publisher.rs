// use async_std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::Arc;

#[derive(Default, Debug)]
pub struct Cache;

impl Cache {
    pub fn clear(&self) {
        log::info!("Cache cleared");
    }
}

/// Interface for publishing messages
#[derive(Default, Debug)]
pub struct Publisher {
    cache: Arc<Cache>,
}

impl Publisher {
    /// Returns a reference to this publisher's cache
    pub fn cache(&self) -> &Arc<Cache> {
        &self.cache
    }
}
