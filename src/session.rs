use crate::{Peer, Subscriptions};
use async_std::{
    sync::{Arc, RwLock, Weak},
    task,
};
use log::info;
use nanoid::nanoid;

/// Represents a client and holds all of its data, may it be active or not.
/// If the client is connected, `peer` is used to retrieve its information and
/// send him packets.
#[derive(Debug)]
pub struct Session {
    id: String,
    client_id: String,
    peer: Weak<Peer>,
    subs: Subscriptions,
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
    pub fn subs(&self) -> &Subscriptions {
        &(self.subs)
    }

    /// Creates a new subcription.
    /// If the topic was already used (replacement), returns true,
    /// otherwise false
    pub fn subscribe(&mut self, topic: &str) -> bool {
        self.subs.add(topic)
    }
}

/// Holds sessions manipulated from the Command Loop
#[derive(Default)]
pub struct Sessions {
    db: Vec<Arc<RwLock<Session>>>,
}

impl Sessions {
    /// Returns the number of sessions
    pub fn len(&self) -> usize {
        self.db.len()
    }

    /// Returns true is the collection is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Searches for the Session at given index and returns it.
    /// If `take`  is set, the session will be extracted from the database
    pub fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>> {
        self.db
            .iter()
            .position(|c| task::block_on(c.read()).client_id() == client_id)
            .map(|index| self.db.swap_remove(index))
    }

    /// Returns the client given its id. If not client exist, returns None
    pub fn get(&self, client_id: &str) -> Option<Arc<RwLock<Session>>> {
        self.db
            .iter()
            .position(|c| task::block_on(c.read()).client_id() == client_id)
            .map(|index| self.db[index].clone())
    }

    /// Add the given session into the database
    pub fn add(&mut self, session: Arc<RwLock<Session>>) {
        self.db.push(session);
    }
}
