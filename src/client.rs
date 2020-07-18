use crate::Peer;
use async_std::sync::{Arc, RwLock, Weak};
use log::debug;
use nanoid::nanoid;

#[derive(Debug)]
pub struct Client {
    id: String,
    peer: Weak<RwLock<Peer>>,
}

impl Client {
    pub fn new(peer: Arc<RwLock<Peer>>) -> Client {
        let id = format!("sage_mqtt-{}", nanoid!());
        debug!("New client: {}", id);
        Client {
            id,
            peer: Arc::downgrade(&peer),
        }
    }
}
