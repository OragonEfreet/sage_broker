//! `sage_broker` library.
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![allow(clippy::large_enum_variant)]

mod broker_settings;
mod command;
mod control;
mod peer;
mod session;
mod sessions;
mod trigger;

/// All functions related to service control.
pub mod service;

pub use broker_settings::BrokerSettings;
use command::Command;
use futures::channel::mpsc;
use peer::Peer;
use sage_mqtt::Packet;
pub use session::Session;
pub use sessions::Sessions;
pub use trigger::Trigger;
/// The MPSC sender for controlling a running server
pub type CommandSender = mpsc::UnboundedSender<Command>;
/// The MPSC sender for controlling a running server
pub type CommandReceiver = mpsc::UnboundedReceiver<Command>;
use control::{Action, Control};

type PacketReceiver = mpsc::UnboundedReceiver<Packet>;
type PacketSender = mpsc::UnboundedSender<Packet>;
