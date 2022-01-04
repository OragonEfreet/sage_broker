use crate::Sessions;
use sage_mqtt::{Publish, ReasonCode};
use std::sync::{Arc, RwLock};

pub async fn run(publish: Publish, sessions: Arc<RwLock<Sessions>>) -> Result<(), ReasonCode> {
    // For now we'll apply the naive way.
    // Loop through sessions and if any subscription apply, send it
    // a publish message
    if let Ok(session) = sessions.read() {
        for session in session.iter().filter(|&session| {
            session
                .subs()
                .read()
                .map_or(false, |s| s.matches(&publish.topic_name))
        }) {
            if let Some(peer) = session.peer() {
                peer.send(Publish { ..publish.clone() }.into());
            }
        }
        Ok(())
    } else {
        Err(ReasonCode::UnspecifiedError)
    }
}
