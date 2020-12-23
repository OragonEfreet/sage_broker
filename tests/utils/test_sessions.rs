use async_std::{
    sync::{Arc, RwLock},
    task,
};
use sage_broker::{Session, SessionsBackEnd};

/// Holds sessions manipulated from the Command Loop
#[derive(Default)]
pub struct TestSessions {
    pub db: Vec<Arc<RwLock<Session>>>,
}

impl SessionsBackEnd for TestSessions {
    /// Searches for the Session at given index and returns it.
    /// If `take`  is set, the session will be extracted from the database
    fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>> {
        if let Some(index) = self
            .db
            .iter()
            .position(|c| task::block_on(c.read()).id == *client_id)
        {
            Some(self.db.swap_remove(index)) // We take it
        } else {
            None
        }
    }

    /// Add the given session into the database
    fn add(&mut self, session: Arc<RwLock<Session>>) {
        self.db.push(session);
    }
}
