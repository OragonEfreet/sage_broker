use crate::{Peer, Subs};
use async_std::sync::{Arc, RwLock, Weak};
use log::info;
use nanoid::nanoid;

/// Represents a client and holds all of its data, may it be active or not.
/// If the client is connected, `peer` is used to retrieve its information and
/// send him packets.
#[derive(Debug)]
pub struct Session {
    id: String,
    client_id: String,
    peer: RwLock<Weak<Peer>>,
    subs: RwLock<Subs>,
}

impl Session {
    /// Creates a new session, giving a peer and an id
    pub fn new(client_id: &str, peer: Arc<Peer>) -> Self {
        let id = format!("session_{}", nanoid!(10));
        info!("New session: Unique ID:{:?}, Client ID:{:?}", id, client_id);

        Session {
            id,
            client_id: client_id.into(),
            peer: RwLock::new(Arc::downgrade(&peer)),
            subs: Default::default(),
        }
    }

    /// A unique ID that cannot be changed neither can collide with
    /// other instances of `Session`
    /// This ID is not part of MQTT specification but is used to ensure a new
    /// session can be created with a same client_id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the client_id of the session
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Assign the session to another peer
    pub async fn bind(&self, peer: Arc<Peer>) {
        *(self.peer.write().await) = Arc::downgrade(&peer);
    }

    /// Gets the currently bound peer as as owning pointer
    pub async fn peer(&self) -> Option<Arc<Peer>> {
        self.peer.read().await.upgrade()
    }

    /// Gets the subscriptions this session has
    pub fn subs(&self) -> &RwLock<Subs> {
        &self.subs
    }
}
