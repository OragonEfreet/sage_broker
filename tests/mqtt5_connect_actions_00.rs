use async_std::task;
use sage_broker::Broker;
use std::{
    io::{prelude::*, BufReader},
    net::TcpStream,
    time::Instant,
};

const TIMEOUT_DELAY: u16 = 3;

/// Requirements:
/// > If the Server does not receive a CONNECT packet within a reasonable amount
/// > of time after the Network Connection is established
/// > the Server SHOULD close the Network Connection.
#[test]
fn server_connect_timeout() {
    let server = task::spawn(async {
        let broker = Broker::default()
            .with_connect_timeout_delay(TIMEOUT_DELAY)
            .start();
        broker.listen("localhost:6788").await.unwrap();
    });

    let stream = TcpStream::connect("localhost:6788").unwrap();
    let mut stream = BufReader::new(stream);

    let now = Instant::now();
    let delay_with_tolerance = (TIMEOUT_DELAY as f32 * 1.5) as u64;

    // Listen loop and sending nothing
    loop {
        let mut recv_buffer = String::new();
        if let Ok(0) = stream.read_line(&mut recv_buffer) {
            break;
        }

        assert!(now.elapsed().as_secs() <= delay_with_tolerance);
    }

    assert!(now.elapsed().as_secs() >= TIMEOUT_DELAY as u64);

    task::block_on(server.cancel());
}
