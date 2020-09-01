use crate::Peer;
use async_std::sync::{Arc, RwLock};
use sage_mqtt::Packet;
use std::fmt;

pub enum Event {
    Control(Arc<RwLock<Peer>>, Packet),
    EndPeer(Arc<RwLock<Peer>>),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Control(_, _) => write!(f, "Control"),
            Event::EndPeer(_) => write!(f, "EndPeer"),
        }
    }
}
