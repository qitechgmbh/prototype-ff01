use std::net::{TcpListener, TcpStream};
use std::io::{self, Write};
use std::time::Duration;
use std::thread;

use crossbeam::channel::{Receiver, TryRecvError};

use crate::telemetry::Payload;

pub fn run(rx: Receiver<Payload>) {
    let listener = TcpListener::bind("0.0.0.0:55667").unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut clients: Vec<TcpStream> = Vec::new();

    loop {
        // 1. Accept new connections
        match listener.accept() {
            Ok((stream, _addr)) => {
                stream.set_nonblocking(true).unwrap();
                clients.push(stream);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // no incoming connections right now
            }
            Err(e) => {
                eprintln!("accept error: {}", e);
            }
        }

        let mut processed = 0;
        const MAX_BATCH: usize = 1000;

        while processed < MAX_BATCH {
            match rx.try_recv() {
                Ok((r_type, data)) => {
                    let data = format!("[{}] {}", r_type.to_str(), data);

                    clients.retain_mut(|stream| {
                        match stream.write_all(data.as_bytes()) {
                            Ok(_) => true,
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => true,
                            Err(_) => false,
                        }
                    });

                    processed += 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }

        // 4. Prevent 100% CPU spin
        thread::sleep(Duration::from_millis(50));
    }
}