use async_std::net::{TcpListener, ToSocketAddrs};
use log::{error, info};

/// Utility function that opens a Tcp connection for listening, returning some
/// `TcpListener` in case of success, `None` otherwise.
/// The function does not perform anything special apart from opening the
/// connexion, meaning you can provide your own instance of `TcpListener` to
/// `listen`.
pub async fn bind(addr: &str) -> Option<TcpListener> {
    let addr = String::from(addr);

    if let Ok(addrs) = addr.to_socket_addrs().await {
        let addrs = addrs
            .map(|addr| addr.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        if let Ok(listener) = TcpListener::bind(addr).await {
            info!("Tcp bound to {}", listener.local_addr().unwrap());
            Some(listener)
        } else {
            error!("Cannot listen from {}", addrs);
            None
        }
    } else {
        error!("Cannot compute addresses from {}", addr);
        None
    }
}
