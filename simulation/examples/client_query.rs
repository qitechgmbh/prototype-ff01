use std::{ io::{Read, Write}, net::TcpStream, time::Duration };

use arrow::ipc::reader::StreamReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap();

    loop {
        std::thread::sleep(Duration::from_millis(2000));

        let request = "weights\n";
        stream.write_all(request.as_bytes()).unwrap();
        stream.flush().unwrap();

        let mut len_buf = [0u8; 8];
        stream.read_exact(&mut len_buf).unwrap();

        let len = u64::from_le_bytes(len_buf) as usize;

        if len == 0 {
            println!("Received no weights");
            continue;
        }

        let mut buf = vec![0u8; len as usize];
        stream.read_exact(&mut buf).unwrap();

        let reader = StreamReader::try_new(&*buf, None).unwrap();

        let mut i = 0;
        let mut j = 0;
        for batch in reader {
            let batch = batch.unwrap();
            i += 1;
            j += batch.num_rows();
        }

        println!("Received {i} batches totaling {j} rows");
    }
}