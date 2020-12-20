//! `sage_broker` library.
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![allow(clippy::large_enum_variant)]

mod broker;
mod broker_settings;
mod command;
mod peer;
mod session;
mod treat;

/// All functions related to service control.
pub mod service;

pub use broker::Broker;
pub use broker_settings::BrokerSettings;
use command::Command;
use futures::channel::mpsc;
use peer::Peer;
use sage_mqtt::Packet;
pub use session::Session;

type CommandSender = mpsc::UnboundedSender<Command>;
type CommandReceiver = mpsc::UnboundedReceiver<Command>;

type PacketReceiver = mpsc::UnboundedReceiver<Packet>;
type PacketSender = mpsc::UnboundedSender<Packet>;
