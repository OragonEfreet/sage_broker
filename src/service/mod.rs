//! The broker service is made of a set of background async tasks.
//!
//! To start a service, you must spawn them or run the utility functions.
//! Either way will require the creation of a `Broker` to personnalise the
//! service.
//!
//! To run a service, call `start` on a bound `TcpListener`.
//! This listener can either be created with `bind` or manually.
//!
//!```no_run
//! # use sage_broker::service;
//! # #[async_std::main]
//! # async fn main() {
//! if let Some(listener) = service::bind("localhost:6788").await {
//!     service::start(listener, Default::default()).await;
//! }
//! # }
//! ```
//!
//! The service will then run and only exit if requested internally or if
//! interrupted by the system.

mod bind;
mod control_loop;
mod listen_peer;
mod listen_tcp;
mod send_loop;
mod start;
pub use bind::bind;
use control_loop::control_loop;
use listen_peer::listen_peer;
use listen_tcp::listen_tcp;
use send_loop::send_loop;
pub use start::start;
