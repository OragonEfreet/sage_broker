use crate::Sessions;
use sage_mqtt::Publish;
use std::sync::{Arc, RwLock};

pub async fn run(publish: Publish, sessions: Arc<RwLock<Sessions>>) {
    // For now we'll apply the naive way.
    // Loop through sessions and if any subscription apply, send it
    // a publish message
    for session in sessions
        .read()
        .unwrap()
        .iter()
        .filter(|&session| session.subs().read().unwrap().matches(&publish.topic_name))
    {
        if let Some(peer) = session.peer() {
            peer.send(Publish { ..publish.clone() }.into());
        }
    }
}
