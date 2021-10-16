use crate::Session;
use async_std::{
    sync::{Arc, RwLock},
    task,
};

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
