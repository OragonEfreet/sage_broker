use crate::{BrokerConfig, Event};
use async_std::{
    future,
    io::BufReader,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    prelude::*,
    task::{self, JoinHandle},
};
use futures::{channel::mpsc, SinkExt};
use log::{error, info};
use sage_mqtt::Packet;
use std::{collections::HashSet, time::Duration};

type EventSender = mpsc::UnboundedSender<Event>;
type EventReceiver = mpsc::UnboundedReceiver<Event>;

pub struct Broker {
    config: BrokerConfig,
    event_sender: EventSender,
    event_receiver: EventReceiver,
    _peers: HashSet<String>,
}

impl Broker {
    pub fn new(addr: &str) -> Self {
        Self::from_config(&BrokerConfig::new(addr))
    }

    pub fn from_config(config: &BrokerConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded();
        Broker {
            config: config.clone(),
            event_sender,
            event_receiver,
            _peers: Default::default(),
        }
    }

    pub fn run(self) -> JoinHandle<()> {
        let mut event_sender = self.event_sender.clone();
        task::spawn(event_loop(
            self.config.timeout_delay,
            self.event_sender.clone(),
            self.event_receiver,
        ));

        let addr = self.config.addr;
        task::spawn(async move {
            if let Ok(addrs) = addr.to_socket_addrs().await {
                info!(
                    "Listening to {}",
                    addrs
                        .map(|addr| addr.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );

                // Listen to any connection
                if let Ok(listener) = TcpListener::bind(addr).await {
                    let mut incoming = listener.incoming();
                    while let Some(stream) = incoming.next().await {
                        match stream {
                            Err(e) => {
                                error!("Cannot accept Tcp stream: {}", e.to_string());
                            }
                            Ok(stream) => {
                                if let Ok(peer_addr) = stream.peer_addr() {
                                    info!("Accepting from {}", peer_addr);
                                    if let Err(e) = event_sender.send(Event::NewPeer(stream)).await
                                    {
                                        error!("{:?}", e);
                                    }
                                } else {
                                    error!("Cannot get peer address");
                                }
                            }
                        }
                    }
                } else {
                    error!("Cannot listen socket");
                }
            } else {
                error!("Cannot compute socket addressed");
            }
        })
    }
}

fn listen_peer(timeout_delay: u16, mut event_sender: EventSender, stream: TcpStream) {
    let peer_addr = stream.peer_addr().unwrap();
    let mut reader = BufReader::new(stream);

    // Create the peer info

    let out_time = Duration::from_secs(timeout_delay as u64);
    task::spawn(async move {
        loop {
            if let Ok(packet) = future::timeout(out_time, Packet::decode(&mut reader)).await {
                match packet {
                    Ok(packet) => {
                        if let Err(e) = event_sender.send(Event::Control(packet)).await {
                            error!("Cannot generate control event: {:?}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        break;
                    }
                }
            } else {
                error!("Time out");
                break;
            }
        }

        info!("Close {}", peer_addr);
    });
}

async fn event_loop(
    timeout_delay: u16,
    event_sender: EventSender,
    mut event_receiver: EventReceiver,
) {
    loop {
        while let Some(event) = event_receiver.next().await {
            match event {
                Event::NewPeer(stream) => {
                    // Start the connection loop for this stream
                    listen_peer(timeout_delay, event_sender.clone(), stream);
                }
                Event::Control(_) => {
                    info!("We got a packet!");
                }
            }
        }
    }
}
