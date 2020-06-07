mod event;
mod event_loop;
mod listen_peer;
mod peer;
mod sender_loop;
mod start;
use event::Event;
use event_loop::event_loop;
use futures::channel::mpsc;
use listen_peer::listen_peer;
use peer::Peer;
use sage_mqtt::Packet;
use sender_loop::sender_loop;
pub use start::start;

type EventSender = mpsc::UnboundedSender<Event>;
type EventReceiver = mpsc::UnboundedReceiver<Event>;

type PacketReceiver = mpsc::UnboundedReceiver<Packet>;
type PacketSender = mpsc::UnboundedSender<Packet>;
