use crate::Session;
use async_std::{
    sync::{Arc, RwLock},
    task,
};

/// Holds sessions manipulated from the Command Loop
#[derive(Default, Clone)]
pub struct Sessions {
    /// NOTE Remove pub afterwards
    pub db: Vec<Arc<RwLock<Session>>>,
}

impl Sessions {
    /// Searches for the Session at given index and returns it.
    /// If `take`  is set, the session will be extracted from the database
    /// NOTE: Unprotected from concurrent call
    pub async fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>> {
        let db = &mut self.db;
        if let Some(index) = db
            .iter()
            .position(|c| task::block_on(c.read()).client_id() == client_id)
        {
            Some(db.swap_remove(index)) // We take it
        } else {
            None
        }
    }

    /// Add the given session into the database
    pub async fn add(&mut self, session: Arc<RwLock<Session>>) {
        self.db.push(session);
    }
}
