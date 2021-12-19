use log::warn;
use sage_mqtt::{defaults, QoS};

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
            retain_enabled: true,
            maximum_packet_size: None,
            topic_alias_maximum: defaults::DEFAULT_TOPIC_ALIAS_MAXIMUM,
            force_keep_alive: false,
        }
    }
}

impl BrokerSettings {
    /// Returns a new instance on default settings restricted to valid options
    /// according to dev current limitations
    pub fn valid_default() -> Self {
        BrokerSettings {
            maximum_qos: QoS::AtMostOnce,
            receive_maximum: 0,
            retain_enabled: false,
            ..Default::default()
        }
    }

    /// Check the settings against current development limitations of the broker.
    /// Returns true only if all the current limitation are satisfied.
    /// This function logs various errors in the currnt settings.
    /// The command loop calls this function and shutdown the server is case of
    /// any invalid configuration option.
    pub fn is_valid(&self) -> bool {
        let mut valid = true;

        if !matches!(self.session_expiry_interval, None | Some(0xFFFFFFFF)) {
            warn!("Invalid Setting value: 'session_expiry_interval': Sessions don't expire yet");
            valid = false;
        }

        if self.maximum_qos != QoS::AtMostOnce {
            warn!("Invalid Setting value: 'maximum_qos': Only QoS Level 0 is supported");
            valid = false;
        }

        if self.receive_maximum > 0 {
            warn!("Invalid Setting value: 'receive_maximum': Only 0 is supported");
            valid = false;
        }

        if self.retain_enabled {
            warn!("Invalid Setting value: 'retain_enabled': retain is not available");
            valid = false;
        }

        if self.maximum_packet_size.is_some() {
            warn!(
                "Invalid Setting value: 'maximum_packet_size': Cannot enforce maximum packet size"
            );
            valid = false;
        }

        if self.topic_alias_maximum > 0 {
            warn!("Invalid Setting value: 'topic_alias_maximum': Topic alias is disabled");
            valid = false;
        }

        valid
    }
}
