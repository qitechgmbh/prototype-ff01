use std::{io::{self, BufRead, BufReader}, net::{TcpListener, TcpStream}, sync::Arc, thread};

use crate::telemetry::Shared;

mod types;
use types::{Request, Response};

pub struct Config {
    pub port: u16
}

pub struct Server {
    listener: TcpListener,
    shared:   Arc<Shared>,
}

impl Server {
    pub fn new(config: Config, shared: Arc<Shared>) -> Result<Self, io::Error> {
        let addr = format!("0.0.0.0:{}", config.port);
        let listener = TcpListener::bind(addr)?;

        Ok(Self { listener, shared })
    }

    pub fn run(self) {
        for stream in self.listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };

            let shared = self.shared.clone();
            thread::spawn(move || Self::handle_client(stream, shared) );
        }
    }

    pub fn handle_client(mut stream: TcpStream, shared: Arc<Shared>) {
        println!("FOUND CLIENT");

        let reader = BufReader::new(stream.try_clone().unwrap());

        for line in reader.lines() {
            println!("LINE: {:?}", line);

            let Ok(line) = line else {
                // connection closed
                break; 
            };

            if line.trim().is_empty() {
                println!("CONTINUE:");
                continue;
            }

            println!("line: {}", &line);
            let request: Request = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    println!("{:?}", e);
                    serde_json::to_writer(&mut stream, &Response::NoSuchRequest).expect("idk");
                    continue;
                }
            };

            match request {
                Request::List => {
                    Self::send_list_response(&mut stream, &shared).expect("AA");
                },
                Request::GetLive { 
                    batch_id, 
                    cursor,
                } => {
                    _ = batch_id;
                    _ = cursor;
                    serde_json::to_writer(&mut stream, &Response::NoData).expect("idk");
                },
                Request::GetRange { from, to } => {
                    _ = from;
                    _ = to;
                    serde_json::to_writer(&mut stream, &Response::InvalidRange).expect("idk");
                },
            }
        }
    }

    fn send_list_response(
        stream: &mut TcpStream,
        shared: &Shared,
    ) -> std::io::Result<()> {
        let registry = shared.segment_registry.load();
        _ = registry;

        let list: Vec<u64> = Default::default();

        let resp = Response::List {
            batches: &list,
        };

        serde_json::to_writer(stream, &resp)?;
        Ok(())
    }
}