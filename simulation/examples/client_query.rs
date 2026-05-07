use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:9000")?;

    loop {
        let request = 
            b"GET /weights HTTP/1.1\r\n\
            Host: 127.0.0.1:9000\r\n\
            Connection: keep-alive\r\n\
            \r\n";

        stream.write_all(request)?;
        stream.flush()?;

        let mut i = 0;  
        loop {
            let mut buf = [0u8; 4096];
            let len = stream.read(&mut buf).unwrap();

            println!("Read {} bytes", len);
            i += len;

            if len == 0 {
                break;
            }

            if len <= 12 {
                let x = std::str::from_utf8(&buf[..len]).unwrap();
                println!("Mini header {} bytes", x);
            }

            println!("Total len: {}", i);
        }

        std::thread::sleep(Duration::from_millis(2000));

        /* 
        // read length prefix (your custom protocol)
        let mut len_buf = [0u8; 8];
        stream.read_exact(&mut len_buf)?;

        let len = u64::from_le_bytes(len_buf) as usize;

        if len == 0 {
            println!("Received no weights");
            continue;
        }

        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;

        let reader = arrow::ipc::reader::StreamReader::try_new(&*buf, None)?;

        let mut batches = 0;
        let mut rows = 0;

        for batch in reader {
            let batch = batch?;
            batches += 1;
            rows += batch.num_rows();
        }

        println!("Received {batches} batches totaling {rows} rows");
        */
    }
}