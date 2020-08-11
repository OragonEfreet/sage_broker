use crate::Peer;
use async_std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub peer: Weak<RwLock<Peer>>,
}

impl Client {
    pub fn new(id: &str, peer: Arc<RwLock<Peer>>) -> Self {
        Client {
            id: id.into(),
            peer: Arc::downgrade(&peer),
        }
    }
}
