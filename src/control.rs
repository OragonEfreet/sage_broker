use crate::Peer;
use async_std::{
    sync::{Arc, RwLock},
    task::JoinHandle,
};
use sage_mqtt::Packet;
use std::fmt;

pub enum Control {
    Packet(Arc<RwLock<Peer>>, Packet),
    RegisterTask(JoinHandle<()>),
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Control")
    }
}

impl From<(Arc<RwLock<Peer>>, Packet)> for Control {
    fn from(v: (Arc<RwLock<Peer>>, Packet)) -> Self {
        Control::Packet(v.0, v.1)
    }
}

impl From<JoinHandle<()>> for Control {
    fn from(v: JoinHandle<()>) -> Self {
        Control::RegisterTask(v)
    }
}
