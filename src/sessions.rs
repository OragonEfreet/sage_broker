use crate::Session;
use async_std::sync::Arc;

/// Holds sessions manipulated from the Command Loop
#[derive(Default, Debug)]
pub struct Sessions {
    db: Vec<Arc<Session>>,
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
    pub fn take(&mut self, client_id: &str) -> Option<Arc<Session>> {
        self.db
            .iter()
            .position(|c| c.client_id() == client_id)
            .map(|index| self.db.swap_remove(index))
    }

    /// Returns the client given its id. If not client exist, returns None
    pub fn get(&self, client_id: &str) -> Option<Arc<Session>> {
        self.db
            .iter()
            .position(|c| c.client_id() == client_id)
            .map(|index| self.db[index].clone())
    }

    /// Add the given session into the database
    pub fn add(&mut self, session: Arc<Session>) {
        self.db.push(session);
    }

    /// Returns an iterator over sessions
    pub fn iter(&self) -> SessionsIterator {
        SessionsIterator {
            inner_it: self.db.iter(),
        }
    }
}

pub struct SessionsIterator<'a> {
    inner_it: std::slice::Iter<'a, Arc<Session>>,
}

impl<'a> Iterator for SessionsIterator<'a> {
    type Item = &'a Arc<Session>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_it.next()
    }
}
