use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;

    // send request (newline-terminated)
    let request = "weights\n";
    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    // read binary response
    let mut buffer = vec![0u8; 4096];

    let n = stream.read(&mut buffer)?;

    println!("received {} bytes", n);

    // optional: decode one row (example)
    let mut i = 0;
    while i < n {
        let flags = buffer[i];
        i += 1;

        println!("flags: {:#08b}", flags);

        if flags & 1 == 0 {
            let ts = i64::from_le_bytes(buffer[i..i+8].try_into()?);
            i += 8;
            println!("timestamp: {}", ts);
        }

        if flags & 2 == 0 {
            let order_id = u32::from_le_bytes(buffer[i..i+4].try_into()?);
            i += 4;
            println!("order_id: {}", order_id);
        }

        if flags & 4 == 0 {
            let w0 = i16::from_le_bytes(buffer[i..i+2].try_into()?);
            i += 2;
            println!("weight_0: {}", w0);
        }

        if flags & 8 == 0 {
            let w1 = i16::from_le_bytes(buffer[i..i+2].try_into()?);
            i += 2;
            println!("weight_1: {}", w1);
        }
    }

    Ok(())
}