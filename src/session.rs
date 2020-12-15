use crate::Peer;
use async_std::sync::{Arc, RwLock, Weak};

/// Represents a client and holds all of its data, may it be active or not.
/// If the client is connected, `peer` is used to retrieve its information and
/// send him packets.
#[derive(Debug)]
pub struct Session {
    /// The client identifier assigned upon connection.
    pub id: String,

    /// The client network information.
    /// This is a non owning shared pointer to `Peer`.
    /// The actual owned peer is created by the tcp listen loop then
    /// held by the peer listen loop.
    pub peer: Weak<RwLock<Peer>>,
}

impl Session {
    /// Builds a new client by giving its id and a shared peer.
    pub fn new(id: &str, peer: Arc<RwLock<Peer>>) -> Self {
        Session {
            id: id.into(),
            peer: Arc::downgrade(&peer),
        }
    }
}