use crate::Peer;
use async_std::sync::{Arc, RwLock, Weak};
use nanoid::nanoid;

/// Represents a client and holds all of its data, may it be active or not.
/// If the client is connected, `peer` is used to retrieve its information and
/// send him packets.
#[derive(Debug)]
pub struct Session {
    id: String,
    client_id: String,
    peer: Weak<RwLock<Peer>>,
}

impl Session {
    /// Creates a new session, giving a peer and an id
    pub fn new(client_id: &str, peer: &Arc<RwLock<Peer>>) -> Self {
        Session {
            id: nanoid!(),
            client_id: client_id.into(),
            peer: Arc::downgrade(peer),
        }
    }

    /// A unique ID that cannot be changed neither can collide with
    /// other instances of `Session`
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the client_id of the session
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Gets the currently bound peer as as owning pointer
    pub fn peer(&self) -> Option<Arc<RwLock<Peer>>> {
        self.peer.upgrade()
    }

    /// Changes the peer of the session.
    /// If a peer was already set, it is unlinked
    pub fn set_peer(&mut self, peer: &Arc<RwLock<Peer>>) {
        self.peer = Arc::downgrade(peer);
    }
}

//Some(format!("sage_mqtt-{}", nanoid!()))
