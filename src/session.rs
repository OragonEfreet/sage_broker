use crate::Peer;
use async_std::sync::{Arc, Weak};
use log::info;
use nanoid::nanoid;
use std::collections::HashSet;

/// Represents a client and holds all of its data, may it be active or not.
/// If the client is connected, `peer` is used to retrieve its information and
/// send him packets.
#[derive(Debug)]
pub struct Session {
    id: String,
    client_id: String,
    peer: Weak<Peer>,
    subs: HashSet<String>,
}

impl Session {
    /// Creates a new session, giving a peer and an id
    pub fn new(client_id: &str, peer: Arc<Peer>) -> Self {
        let id = format!("session_{}", nanoid!(10));
        info!("New session: Unique ID:{:?}, Client ID:{:?}", id, client_id);

        Session {
            id,
            client_id: client_id.into(),
            peer: Arc::downgrade(&peer),
            subs: Default::default(),
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
    pub fn peer(&self) -> Option<Arc<Peer>> {
        self.peer.upgrade()
    }

    /// Changes the peer of the session.
    /// If a peer was already set, it is unlinked
    pub fn set_peer(&mut self, peer: Arc<Peer>) {
        self.peer = Arc::downgrade(&peer);
    }

    /// Returns the list of subscriptions
    pub fn subs(&self) -> &HashSet<String> {
        &(self.subs)
    }

    /// Creates a new subcription.
    /// If the topic was already used (replacement), returns false,
    /// otherwise true
    pub fn subscribe(&mut self, topic: &str) -> bool {
        self.subs.insert(topic.into())
    }
}
