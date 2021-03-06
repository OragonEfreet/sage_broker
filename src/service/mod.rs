//! The broker service is made of a set of background async tasks.
//!
//! The sage broker service is ran by calling `service::run` which is responsible
//! for spawning all asynchronous tasks and waiting for their safe shut down
//! at the end.
//!
//! # Loops
//!
//! ## Command Loop
//!
//! The higher level task is `command_loop`.
//! Command loop is the first executed task and the last terminated.
//! It holds the command channel receiver. As long as at least one command
//! channel sender is open, the loop stays.
//! The job of the command loop is to process and dispatch any packet received
//! from any peer.
//!
//! > When all CommandSender instances are closed, the command loop ends.
//!
//! ## Listen TCP
//!
//! The Listen TCP loop runs as long as the Broker is not set to a closing state.
//! It holds an instance of the command channel sender.
//! Each time a valid TCP connection is made, the loop creates a new Peer
//! instance which invokes the creations of:
//! - A new packet channel used to send packets to the peer stream
//! - A send_peer task which receives from the packet channel
//! - A listen_peer task which receives from the stream
//! - A Peer object which is held by the listen_peer task
//!
//! > When the broker is marked as shut down, the listen tcp loop ends.
//! > This operation will drop:
//! > - A command channel sender
//!
//! ## Listen peer
//!
//! One listen peer exist per active connexion. This loop is created by the
//! listen tcp loop.
//! The listen peer holds an instance of command sender. It parses any incoming
//! data from the stream and send to the command loop.
//! It also holds an owning reference to the Peer instance, sending a copy to
//! the command channel from time to time.
//!
//! > When the peer is marked as closed, the listen peer loop ends.
//! > This action will drop:
//! > - A command channel sender
//! > - The associated Peer
//! The loop has a timeout that will ask for closing the peer if it does not
//! receive incoming data from the stream in a given amount of time.
//!
//! The associated Peer is generaly the only instance. But the list peer loop
//! is able to clone it and send it to the command channel at any time.
//! It means that if the listen peer loop ends and there is still pending
//! operations for that peer, the associated peer won't be closed until those
//! operations are complete.
//!
//! ## Send peer
//! The Send peer task is the writing half of a peer stream. It waits for any
//!
//! incoming packets from the packet channel and serializes it before sending it
//! through TCP.
//! There is one instance of the send peer loop per active peer.
//! > When all PacketSender instances have been closed, the sender peer loop ends.
//!
//! # Safe Close
//!
//! The safe close is initiated by calling `shutdown` on the Broker object.
//! At that moment, the following performs:
//! - The Listen TCP loop ends, releasing a Command Sender
//! - The Command Loop ends.
//!
mod command_loop;
mod listen_peer;
mod listen_tcp;
mod send_peer;
pub use command_loop::command_loop;
use listen_peer::listen_peer;
pub use listen_tcp::listen_tcp;
use send_peer::send_peer;
