use nanoid::nanoid;
use sage_mqtt::{defaults, ConnAck, Connect, QoS, ReasonCode};
use std::cmp::min;

#[derive(Clone, Debug)]
pub struct Broker {
    // pub addr: String,
    pub session_expiry_interval: Option<u32>,
    pub force_session_expiry_interval: bool,
    pub receive_maximum: u16,
    pub maximum_qos: QoS,
    pub retain_enabled: bool,
    pub maximum_packet_size: Option<u32>,
    pub topic_alias_maximum: u16,
    pub keep_alive: u16,
    pub force_keep_alive: bool,
}

impl Default for Broker {
    fn default() -> Self {
        Broker {
            // addr: "localhost:6788".into(),
            keep_alive: defaults::DEFAULT_KEEP_ALIVE,
            session_expiry_interval: defaults::DEFAULT_SESSION_EXPIRY_INTERVAL,
            force_session_expiry_interval: false,
            receive_maximum: defaults::DEFAULT_RECEIVE_MAXIMUM,
            maximum_qos: defaults::DEFAULT_MAXIMUM_QOS,
            retain_enabled: false,
            maximum_packet_size: None,
            topic_alias_maximum: defaults::DEFAULT_TOPIC_ALIAS_MAXIMUM,
            force_keep_alive: false,
        }
    }
}

impl Broker {
    /// Generates an acknowledgement packet given the `connect` packet and the
    /// current configuration.
    pub fn acknowledge_connect(&self, connect: Connect) -> ConnAck {
        // If the server forces the value, we use it.
        // Otherwise we take the value from the connect request or
        // the server one if absent.
        let session_expiry_interval = {
            if self.force_session_expiry_interval {
                self.session_expiry_interval
            } else {
                connect
                    .session_expiry_interval
                    .or(self.session_expiry_interval)
            }
        };

        // The maximum receive_maximum if the minimum value between server and
        // client requirements
        let receive_maximum = min(connect.receive_maximum, self.receive_maximum);

        // The maximum packet size is either the minimal value of client and
        // server requirements or the only value defined by one of them, or None
        let maximum_packet_size = match (connect.maximum_packet_size, self.maximum_packet_size) {
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
        let topic_alias_maximum = min(self.topic_alias_maximum, connect.topic_alias_maximum);

        // Values directly defined by the configuration
        let maximum_qos = self.maximum_qos;
        let retain_available = self.retain_enabled;

        // The keep alive value is given by the connect packet but
        // can be force override by the server
        let keep_alive = if self.force_keep_alive {
            Some(self.keep_alive)
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
}
