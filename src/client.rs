use crate::{Peer, ClientStatus};
use async_std::sync::{Arc, RwLock, Weak};
use log::debug;
use nanoid::nanoid;

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub peer: Weak<RwLock<Peer>>,
    pub status: ClientStatus,
}

impl Client {
    pub fn new(peer: Arc<RwLock<Peer>>) -> Client {
        let id = format!("sage_mqtt-{}", nanoid!());
        debug!("New client: {}", id);
        Client {
            id,
            peer: Arc::downgrade(&peer),
            status: ClientStatus::Connecting,
        }
    }
}
