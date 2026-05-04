use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream, time::Duration,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;
    stream.set_read_timeout(Some(Duration::from_millis(200)))?;

    let i = get_weights(&mut stream)?;
    println!("Counted weights: {}", i);

    Ok(())
}

fn get_weights(stream: &mut TcpStream) -> Result<u32, Box<dyn std::error::Error>> {
    // send request (newline-terminated)
    let request = "weights\n";
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    let mut i = 0;
    loop {
        let mut flags_buf = [0u8; 1];
        if let Err(e) = stream.read_exact(&mut flags_buf) {
            if e.kind() == ErrorKind::WouldBlock {
                break;
            } 

            return Err(Box::new(e));
        };

        let flags = flags_buf[0];

        let mut ts_buf = [0u8; 8];
        stream.read_exact(&mut ts_buf)?;

        let mut buf_4b = [0u8; 4];
        let mut buf_2b = [0u8; 2];

        if flags & 1 == 0 {
            stream.read_exact(&mut buf_4b)?;
        }

        if flags & 2 == 0 {
            stream.read_exact(&mut buf_2b)?;
        }

        if flags & 4 == 0 {
            stream.read_exact(&mut buf_2b)?;
        }

        i += 1;
    }

    Ok(i)
}

fn get_plates(stream: &mut TcpStream) -> Result<u32, Box<dyn std::error::Error>> {
    // send request (newline-terminated)
    let request = "weights\n";
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    let mut i = 0;
    loop {
        let mut flags_buf = [0u8; 1];
        if let Err(e) = stream.read_exact(&mut flags_buf) {
            if e.kind() == ErrorKind::WouldBlock {
                break;
            } 

            return Err(Box::new(e));
        };

        let flags = flags_buf[0];

        let mut ts_buf = [0u8; 8];
        stream.read_exact(&mut ts_buf)?;

        let mut buf_4b = [0u8; 4];
        let mut buf_2b = [0u8; 2];

        if flags & 1 == 0 {
            stream.read_exact(&mut buf_4b)?;
        }

        if flags & 2 == 0 {
            stream.read_exact(&mut buf_2b)?;
        }

        if flags & 4 == 0 {
            stream.read_exact(&mut buf_2b)?;
        }

        i += 1;
    }

    Ok(i)
}