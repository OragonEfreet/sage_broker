//! `sage_broker` library.
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![allow(clippy::large_enum_variant)]

use async_std::{channel, sync::Arc};

mod broker_settings;
//mod command;
mod control;
mod peer;
mod session;
mod sessions;
mod subs;
mod trigger;

/// All functions related to service control.
pub mod service;

pub use broker_settings::BrokerSettings;
//use command::Command;
use peer::Peer;
use sage_mqtt::Packet;
pub use session::Session;
pub use sessions::Sessions;
pub use subs::Subs;
pub use trigger::Trigger;
/// The MPSC sender for controlling a running server
pub type CommandSender = channel::Sender<(Arc<Peer>, Packet)>;
/// The MPSC sender for controlling a running server
pub type CommandReceiver = channel::Receiver<(Arc<Peer>, Packet)>;

type PacketReceiver = channel::Receiver<Packet>;
type PacketSender = channel::Sender<Packet>;
