use std::{io::{BufRead, BufReader}, net::{TcpListener, TcpStream}, sync::Arc, thread};

use serde::{Deserialize, Serialize};

use crate::telemetry::{Shared, binary::BatchReadCursor};

pub type TimeDate = [u8; 23]; // 2026-04-22T14:33:12.123

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// List all available batches
    List,

    /// Retrieve data from cache for next batch
    GetLive { batch_id: u64, cursor: Option<BatchReadCursor> },

    /// Retrieve batches in a range
    GetRange { from: u64, to: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    List { batches: Vec<u64> },
    Live {
        batch_id: u64,
        cursor: BatchReadCursor,
        data: Vec<u8>,
    },
    NoData,
    OutOfSync { current_batch_id: u64 },
    Range { batches: Vec<u8> },
    InvalidRequest,
}

pub fn run(shared: Arc<Shared>) {
    let listener = TcpListener::bind("0.0.0.0:9000")
        .expect("failed to bind");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        let shared = shared.clone();
        thread::spawn(move || handle_client(stream, shared) );
    }
}

pub fn handle_client(stream: TcpStream, shared: Arc<Shared>) {
    let reader = BufReader::new(stream);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break, // connection dropped
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(_) => {
                // optionally log malformed request
                continue;
            }
        };


    }
}