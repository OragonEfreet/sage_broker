//! `sage_broker` library.
#![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![allow(clippy::large_enum_variant)]

mod broker;
mod broker_settings;
mod client;
mod control;
mod peer;

/// All functions related to service control.
pub mod service;

pub use broker::Broker;
pub use broker_settings::BrokerSettings;
pub use client::Client;
use control::Control;
use futures::channel::mpsc;
use peer::Peer;
use sage_mqtt::Packet;

type ControlSender = mpsc::UnboundedSender<Control>;
type ControlReceiver = mpsc::UnboundedReceiver<Control>;

type PacketReceiver = mpsc::UnboundedReceiver<Packet>;
type PacketSender = mpsc::UnboundedSender<Packet>;
