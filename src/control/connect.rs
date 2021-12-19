use crate::{BrokerSettings, Peer, Session, Sessions};
use async_std::sync::{Arc, RwLock};
use sage_mqtt::{Connect, Disconnect, ReasonCode};

pub async fn run(
    connect: Connect,
    sessions: Arc<RwLock<Sessions>>,
    settings: Arc<BrokerSettings>,
    peer: Arc<Peer>,
) {
    // First, we prepare an first connack using broker policy
    // and infer the actual client_id requested for this client
    let mut connack = settings.acknowledge_connect(&connect);

    if connack.reason_code == ReasonCode::Success {
        let client_id = connack
            .assigned_client_id
            .clone()
            .or(connect.client_id)
            .unwrap();

        let mut sessions = sessions.write().await;

        let clean_start = connect.clean_start;
        // Session creation/overtaking
        // First, we get the may be existing session from the db:
        // TODO: This can be simplified
        let session = {
            if let Some(session) = sessions.take(&client_id) {
                // If the existing session has a peer, it'll be disconnected with takeover
                if let Some(peer) = session.read().await.peer() {
                    peer.send_close(
                        Disconnect {
                            reason_code: ReasonCode::SessionTakenOver,
                            ..Default::default()
                        }
                        .into(),
                    )
                    .await;
                }

                if clean_start {
                    connack.session_present = false;
                    let client_id = client_id.clone();
                    Arc::new(RwLock::new(Session::new(&client_id, peer.clone())))
                } else {
                    connack.session_present = true;
                    session.write().await.set_peer(peer.clone());
                    session
                }
            } else {
                connack.session_present = false;
                let client_id = client_id.clone();
                Arc::new(RwLock::new(Session::new(&client_id, peer.clone())))
            }
        };
        sessions.add(session.clone());
        peer.bind(session).await;
        peer.send(connack.into()).await;
    } else {
        peer.send_close(connack.into()).await;
    }
}
