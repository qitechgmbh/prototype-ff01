use std::{io::Read, net::TcpStream};

use telemetry_core::Event;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:9001")?;

    let mut buf = [0u8; 512];
    loop {
        stream.read_exact(&mut buf[0..2]).unwrap();
        let len = u16::from_le_bytes(buf[0..2].try_into().unwrap()) as usize;

        stream.read_exact(&mut buf[0..len]).unwrap();
        let data = &buf[0..len];

        let event = Event::decode(data);
        
        println!("Received: {:?}", &event);
    }
}