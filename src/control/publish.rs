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
        dbg!(session);
        // if session.subs().read().await.matches(&publish.topic_name) {
        //     println!("Matches");
        // } else {
        //     log::error!("Bo-hou");
        // }
    }
}
