use crate::{Action, BrokerSettings, Control, Peer, Session, Sessions};
use async_std::sync::{Arc, RwLock};
use async_trait::async_trait;
use sage_mqtt::{Connect, Disconnect, ReasonCode};

#[async_trait]
impl Control for Connect {
    async fn control(
        self,
        sessions: Arc<RwLock<Sessions>>,
        settings: &Arc<BrokerSettings>,
        peer: &Arc<RwLock<Peer>>,
    ) -> Action {
        // First, we prepare an first connack using broker policy
        // and infer the actual client_id requested for this client
        let mut connack = settings.acknowledge_connect(&self);

        if connack.reason_code == ReasonCode::Success {
            let client_id = connack
                .assigned_client_id
                .clone()
                .or(self.client_id)
                .unwrap();

            let mut sessions = sessions.write().await;

            let clean_start = self.clean_start;
            // Session creation/overtaking
            // First, we get the may be existing session from the db:
            let session = {
                if let Some(session) = sessions.take(&client_id) {
                    // If the existing session has a peer, it'll be disconnected with takeover
                    if let Some(peer) = session.read().await.peer() {
                        peer.write()
                            .await
                            .send_close(
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
                        Arc::new(RwLock::new(Session::new(&client_id, peer)))
                    } else {
                        connack.session_present = true;
                        session.write().await.set_peer(peer);
                        session
                    }
                } else {
                    connack.session_present = false;
                    let client_id = client_id.clone();
                    Arc::new(RwLock::new(Session::new(&client_id, peer)))
                }
            };
            sessions.add(session.clone());
            peer.write().await.bind(session);

            Action::Respond(connack.into())
        } else {
            Action::RespondAndDisconnect(connack.into())
        }
    }
}
