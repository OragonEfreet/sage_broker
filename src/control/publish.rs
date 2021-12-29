use crate::Sessions;
use async_std::{
    sync::{Arc, RwLock},
    task,
};
use sage_mqtt::Publish;

pub async fn run(publish: Publish, sessions: Arc<RwLock<Sessions>>) {
    // For now we'll apply the naive way.
    // Loop through sessions and if any subscription apply, send it
    // a publish message
    for session in sessions
        .read()
        .await
        .iter()
        .filter(|&session| task::block_on(session.subs().read()).matches(&publish.topic_name))
    {
        if let Some(peer) = session.peer().await {
            peer.send(Publish { ..publish.clone() }.into()).await;
        }
    }
}
