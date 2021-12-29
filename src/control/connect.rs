use crate::{BrokerSettings, Cache, Peer, Session, Sessions};
use async_std::sync::{Arc, RwLock};
use nanoid::nanoid;
use sage_mqtt::{ConnAck, Connect, Disconnect, ReasonCode};
use std::cmp::min;

pub async fn run(
    settings: Arc<BrokerSettings>,
    sessions: Arc<RwLock<Sessions>>,
    connect: Connect,
    peer: Arc<Peer>,
    cache: Arc<Cache>,
) {
    // First, we prepare an first connack using broker policy
    // and infer the actual client_id requested for this client
    let mut connack = acknowledge_connect(settings, &connect);

    if connack.reason_code == ReasonCode::Success {
        let client_id = connack
            .assigned_client_id
            .clone()
            .or(connect.client_id)
            .unwrap();

        let clean_start = connect.clean_start;
        // Session creation/overtaking
        // First, we get the may be existing session from the db:
        // TODO: This can be simplified
        let session = {
            if let Some(session) = sessions.write().await.take(&client_id) {
                // If the existing session has a peer, it'll be disconnected with takeover
                if let Some(peer) = session.peer().await {
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
                    Arc::new(Session::new(&client_id, peer.clone(), cache))
                } else {
                    connack.session_present = true;
                    session.bind(peer.clone()).await;
                    session
                }
            } else {
                connack.session_present = false;
                Arc::new(Session::new(&client_id, peer.clone(), cache))
            }
        };
        sessions.write().await.add(session.clone());
        peer.bind(session).await;
        peer.send(connack.into()).await;
    } else {
        peer.send_close(connack.into()).await;
    }
}

fn acknowledge_connect(settings: Arc<BrokerSettings>, connect: &Connect) -> ConnAck {
    // If the server forces the value, we use it.
    // Otherwise we take the value from the connect request or
    // the server one if absent.
    let session_expiry_interval = {
        if settings.force_session_expiry_interval {
            settings.session_expiry_interval
        } else {
            connect
                .session_expiry_interval
                .or(settings.session_expiry_interval)
        }
    };

    // The maximum receive_maximum if the minimum value between server and
    // client requirements
    let receive_maximum = min(connect.receive_maximum, settings.receive_maximum);

    // The maximum packet size is either the minimal value of client and
    // server requirements or the only value defined by one of them, or None
    let maximum_packet_size = match (connect.maximum_packet_size, settings.maximum_packet_size) {
        (Some(a), Some(b)) => Some(min(a, b)),
        (None, Some(a)) | (Some(a), None) => Some(a),
        _ => None,
    };

    // If the client did not specify a client ID, the server must generate
    // and assign one.
    let assigned_client_id = if connect.client_id.is_some() {
        None
    } else {
        Some(format!("sage_mqtt-{}", nanoid!()))
    };

    // The topic alias maximum if the minimum value between server and client
    // requirements.
    let topic_alias_maximum = min(settings.topic_alias_maximum, connect.topic_alias_maximum);

    // Values directly defined by the configuration
    let maximum_qos = settings.maximum_qos;
    let retain_available = settings.retain_enabled;

    // The keep alive value is given by the connect packet but
    // can be force overriden by the server
    let keep_alive = if settings.force_keep_alive {
        Some(settings.keep_alive)
    } else {
        None
    };

    // Enhanced authentication is not supported for now
    let (reason_code, reason_string) = {
        if connect.authentication.is_some() || connect.user_name.is_some() {
            (
                ReasonCode::BadAuthenticationMethod,
                Some("Enhanced anthentication non supported".into()),
            )
        } else {
            (ReasonCode::Success, None)
        }
    };

    let wildcard_subscription_available = false;
    let subscription_identifiers_available = false;
    let shared_subscription_available = false;
    let response_information = None;
    let reference = None;

    let session_present = false;
    let user_properties = Default::default();
    let authentication = None;

    ConnAck {
        reason_code,
        session_expiry_interval,
        receive_maximum,
        maximum_qos,
        retain_available,
        maximum_packet_size,
        assigned_client_id,
        topic_alias_maximum,
        reason_string,
        wildcard_subscription_available,
        subscription_identifiers_available,
        shared_subscription_available,
        keep_alive,
        response_information,
        reference,
        session_present,
        user_properties,
        authentication,
    }
}
