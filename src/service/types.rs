use std::time::Duration;

use serde::Serialize;
use crossbeam::channel::TryRecvError;

use super::State;

#[derive(Debug, Clone)]
pub struct Config {
    pub config_path: String,
    pub reconnect_attempts_max: u32,
    pub timeout_reconnect: Duration,
    pub timeout_heartbeat: Duration,
    pub timeout_sending:      Duration,
}

#[derive(Debug, Clone, Serialize)]
pub struct TargetRange {
    pub min:     f64,
    pub max:     f64,
    pub desired: f64,
}

impl TargetRange {
    pub fn in_bounds(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub doc_entry: i32,
    pub line_number: i32,
    pub item_code: String,
    pub whs_code: String,
    pub weight_bounds: TargetRange,
}

#[derive(Debug)]
pub enum Request {
    Terminate,
    UpdateState(State, u32),
}

#[derive(Debug)]
pub enum Response {
    NextState(State),
    Error(anyhow::Error),
}

pub type ServiceSender   = crossbeam::channel::Sender<Request>;
pub type ServiceReceiver = crossbeam::channel::Receiver<Response>;

pub type WorkerSender    = crossbeam::channel::Sender<Response>;
pub type WorkerReceiver  = crossbeam::channel::Receiver<Request>;

#[derive(Debug)]
pub struct Connection {
    sender:   ServiceSender, 
    receiver: ServiceReceiver, 
    pending:  bool,
}

impl Connection {
    pub fn new(sender: ServiceSender, receiver: ServiceReceiver) -> Self {
        Self { sender, receiver, pending: false }
    }

    pub fn has_pending(&self) -> bool {
        self.pending
    }

    pub fn recv(&mut self) -> Option<Response> {
        if !self.pending { panic!("Not allowed") }

        let result = self.receiver.try_recv();

        match result {
            Ok(response) => {
                // wait for response before allowing new requests
                self.pending = false;
                return Some(response);
            }
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                panic!("Worker disconnected but isn't supposed to!");
            }
        }
    }

    pub fn send(&mut self, state: State, plate_count: u32) {
        if self.pending { panic!("Not allowed") }

        self.sender.try_send(Request::UpdateState(state, plate_count))
            .expect("Should not be full and not disconnected");

        self.pending = true;
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Err(e) = self.sender.try_send(Request::Terminate) {
            println!("CRITICAL ERROR WHILE DROPPING CONNECIION: {}", e);
        }
    }
}