use nanoid::nanoid;
use sage_mqtt::{defaults, ConnAck, Connect, QoS, ReasonCode};
use std::cmp::min;

/// Configuration structure for a broker.
/// This structure is used to customize the behaviour of your broker. It is used
/// as a parameter for the `start` function.
#[derive(Clone, Debug)]
pub struct BrokerSettings {
    /// Once the connection is closed, the client and server still keep the
    /// session active during a certain amount of time expressed in seconds.
    /// - If the value is `0` (default) the session ends when the connection is closed.
    /// - If the value is `0xFFFFFFFF` the session never expires.
    /// The client can override the session expiry interval within the
    /// DISCONNECT packet.
    pub session_expiry_interval: Option<u32>,

    /// if `true` the server will always override the session expiry interval
    /// requested by the client. If false (default) the client's value is used
    /// or, if absent, the server one.
    pub force_session_expiry_interval: bool,

    /// This value sets the maximum number of _AtLeastOnce_ and _ExactlyOnce_
    /// packets that should be processed concurrently.
    /// There is no such limit for QoS `AtMostOnce` packets.
    /// The default value is `65_535`
    /// Upon receiving a connect packet, the assigned received maximum will be
    /// the lowest value between the server configuration and the client request
    pub receive_maximum: u16,

    /// The maximum quality of service the server is willing to operate on.
    pub maximum_qos: QoS,

    /// If `true` the server will allow retain messages. Default is `false`
    pub retain_enabled: bool,

    /// Defines the maximum size per packet the client is willing to receive
    /// from the server. It is a procotol error to send a packet which size
    /// exceeds this value and the client is expected to disconnect from the
    /// server with a `PacketTooLarge` error.
    /// This value cannot be `0`. Sending or receiving a CONNECT packet with a
    /// `maximum_packet_size` of value `0` is a procotol error.
    /// `maximum_packet_size` is `None` (default), there is no size limit.
    /// The maximum packet size is either the minimal value of client and
    /// server requirements or the only value defined by one of them, or None.
    pub maximum_packet_size: Option<u32>,

    /// Topic aliases are a way to reduce the size of packets by substituting
    /// aliases (which are strings) to integer values.
    /// The number of aliases allowed by the server is defined
    /// with the `topic_alias_maximum`. It can be `0`, meaning aliases are
    /// entirely disallowed.
    /// The assigned topic alias maximum if the minimum value between server
    /// and client requirements.
    pub topic_alias_maximum: u16,

    /// Specifies the maximum amount of time the client and the server may not
    /// communicate with each other. This value is expressed in seconds.
    /// If the server does not receive any packet from the client in one and
    /// a half times this interval, it will close the connection. Likewise, the
    /// client will close the connection under the same condition. The default
    /// keep alive value is `600` (10mn).
    /// Not that the keep alive mechanism is deactivated if the value is `0`.
    pub keep_alive: u16,

    /// If `true` the connections will use the keep alive value from the server.
    /// if `false` the value requested by the client will be use instead.
    pub force_keep_alive: bool,
}

impl Default for BrokerSettings {
    fn default() -> Self {
        BrokerSettings {
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

impl BrokerSettings {
    /// Generates an acknowledgement packet given the `connect` packet and the
    /// current configuration.
    pub fn acknowledge_connect(&self, connect: &Connect) -> ConnAck {
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
        // can be force overriden by the server
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
