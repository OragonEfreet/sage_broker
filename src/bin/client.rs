use async_std::task;
use sage_mqtt::ControlPacket;
use std::io::prelude::*;
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6788")?;

    let mut buffer = Vec::new();

    let packet = ControlPacket::Connect(Default::default());
    let encoded = task::block_on(packet.encode(&mut buffer));

    if let Ok(n) = encoded {
        println!("Sending {} bytes", n);
        stream.write_all(&buffer)?;
    } else {
        println!(">> {:?}", encoded);
    }

    Ok(())
} // the stream is closed here
