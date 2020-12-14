use async_std::{
    io::{prelude::*, Cursor},
    net::TcpStream,
    task,
};
use sage_mqtt::{Connect, Packet};
use std::{thread, time::Duration};

fn connect() -> TcpStream {
    for _ in 0u8..5u8 {
        log::info!("Connecting...");
        if let Ok(stream) = task::block_on(TcpStream::connect("localhost:6788")) {
            return stream;
        }

        thread::sleep(Duration::from_secs(1));
    }
    panic!("Cannot connect");
}

fn main() {
    pretty_env_logger::init();
    let mut stream = connect();

    task::block_on(async {
        // Send an invalid connect packet and wait for an immediate disconnection
        // from the server.
        let packet = Packet::from(Connect {
            authentication: Some(Default::default()),
            ..Default::default()
        });
        let mut buffer = Vec::new();
        packet.encode(&mut buffer).await.unwrap();

        log::info!("Sending connect packet");
        while stream.write(&buffer).await.is_err() {}

        loop {
            let mut buf = vec![0u8; 1024];
            let result = stream.read(&mut buf).await;
            match result {
                Err(e) => {
                    log::error!("{:?}", e);
                    break;
                }
                Ok(0) => {
                    log::info!("Server disconnected");
                    break;
                }
                Ok(_) => {
                    let mut buf = Cursor::new(buf);
                    if let Ok(packet) = Packet::decode(&mut buf).await {
                        log::info!("Received {:?}", packet);
                    } else {
                        log::error!("Cannot decode packet");
                        break;
                    }
                }
            }
        }
    });
}
