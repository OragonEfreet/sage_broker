use crate::Peer;
use async_std::sync::{Arc, RwLock};
use sage_mqtt::Packet;
use std::fmt;

// It's an enum because we will want to create other command types
pub enum Command {
    Control(Arc<RwLock<Peer>>, Packet),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command")
    }
}

impl From<(Arc<RwLock<Peer>>, Packet)> for Command {
    fn from(v: (Arc<RwLock<Peer>>, Packet)) -> Self {
        Command::Control(v.0, v.1)
    }
}
