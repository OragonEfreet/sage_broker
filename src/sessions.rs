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
    /// Searches for the Session at given index and returns it.
    /// If `take`  is set, the session will be extracted from the database
    pub fn take(&mut self, client_id: &str) -> Option<Arc<RwLock<Session>>> {
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
    pub fn add(&mut self, session: Arc<RwLock<Session>>) {
        self.db.push(session);
    }
}

/*
// We search, in any other aleady existing sessions, if the name is
// already taken. If so, we extract the client.
let client = {
    // If a session exists for the same client id
    if let Some(index) = sessions.iter().position(|c| c.id == client_id) {
        let client = sessions.swap_remove(index); // We take it

        // If the session already has a peer
        if let Some(peer) = client.peer.upgrade() {
            let packet = Disconnect {
                reason_code: ReasonCode::SessionTakenOver,
                ..Default::default()
            };
            peer.write().await.send_close(packet.into()).await;
        }
        client
    } else {
        Arc::new(Session::new(&client_id, peer.clone()))
    }
};

// Now we create the new client and push it into the collection.
sessions.push(client.clone());
info!("New client: {}", client.id);
*/
