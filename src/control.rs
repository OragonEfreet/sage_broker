use crate::Peer;
use async_std::sync::{Arc, RwLock};
use sage_mqtt::Packet;
use std::fmt;

pub struct Control(pub Arc<RwLock<Peer>>, pub Packet);

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Control")
    }
}

impl From<(Arc<RwLock<Peer>>, Packet)> for Control {
    fn from(v: (Arc<RwLock<Peer>>, Packet)) -> Self {
        Control(v.0, v.1)
    }
}
