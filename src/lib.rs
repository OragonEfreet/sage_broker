//! `sage_broker` library.
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![allow(clippy::large_enum_variant)]

mod broker_settings;
mod command;
mod peer;
mod session;
mod sessions;
mod sessions_back_end;
mod treat;
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
pub use sessions_back_end::SessionsBackEnd;
pub use trigger::Trigger;
/// bim
pub type CommandSender = mpsc::UnboundedSender<Command>;
/// bam
pub type CommandReceiver = mpsc::UnboundedReceiver<Command>;

type PacketReceiver = mpsc::UnboundedReceiver<Packet>;
type PacketSender = mpsc::UnboundedSender<Packet>;
