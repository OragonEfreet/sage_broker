//! `sage_broker` library.
// #![warn(missing_docs)]
// #![warn(missing_doc_code_examples)]
#![allow(clippy::large_enum_variant)]

mod broker;
mod client;
mod event;
mod peer;
pub mod service;
pub use broker::Broker;
pub use client::Client;
use event::Event;
use futures::channel::mpsc;
use peer::Peer;
use sage_mqtt::Packet;

type EventSender = mpsc::UnboundedSender<Event>;
type EventReceiver = mpsc::UnboundedReceiver<Event>;

type PacketReceiver = mpsc::UnboundedReceiver<Packet>;
type PacketSender = mpsc::UnboundedSender<Packet>;
