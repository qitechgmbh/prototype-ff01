use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread};

pub fn run() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565")?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    println!("Client connected: {:?}", stream.peer_addr());

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(len) => {
            let request = &buffer[0..len];

            if &request[0..8] == b"GET_LIST" {
                println!("GET_LIST!");
                let _ = stream.write_all(b"OMG A LIST");
            }

            else if &request[0..9] == b"GET_ENTRY" {
                println!("GET_ENTRY!");
                let _ = stream.write_all(b"OMG AN ENTRY");
            }

            else {
                let _ = stream.write_all(b"Invalid Request");
            }
        }
        Err(e) => {
            eprintln!("Failed to read request: {}", e);
            return;
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn execute() {
        _ = super::run();
    }
}