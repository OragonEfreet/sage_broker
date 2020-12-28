use async_std::{
    sync::{Arc, RwLock},
    task,
};
use async_trait::async_trait;
use sage_broker::{Session, SessionsBackEnd};

type DatabaseT = Vec<Arc<RwLock<Session>>>;
/// Holds sessions manipulated from the Command Loop
#[derive(Default, Clone)]
pub struct TestSessions {
    pub db: Arc<RwLock<DatabaseT>>,
}

#[async_trait]
impl SessionsBackEnd for TestSessions {
    /// Searches for the Session at given index and returns it.
    /// If `take`  is set, the session will be extracted from the database
    async fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>> {
        let mut db = self.db.write().await;
        if let Some(index) = db
            .iter()
            .position(|c| task::block_on(c.read()).id == *client_id)
        {
            Some(db.swap_remove(index)) // We take it
        } else {
            None
        }
    }

    /// Add the given session into the database
    async fn add(&mut self, session: Arc<RwLock<Session>>) {
        self.db.write().await.push(session);
    }
}
