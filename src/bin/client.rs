use async_std::task;
use sage_mqtt::Packet;
use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6788")?;

    for _ in 0..2 {
        let mut buffer = Vec::new();
        let packet = Packet::Publish(Default::default());

        if let Ok(n) = task::block_on(packet.encode(&mut buffer)) {
            println!("Sending {} bytes", n);
            stream.write_all(&buffer)?;
        }
    }
    println!("Pause...");
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("Done");

    Ok(())
} // the stream is closed here
