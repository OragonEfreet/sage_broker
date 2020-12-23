mod send_waitback;
mod server;
pub use send_waitback::send_waitback;
pub use server::TestServer;

pub const TIMEOUT_DELAY: u16 = 3;
