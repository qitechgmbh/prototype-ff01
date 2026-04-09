use beas_bsl::Client;

use crate::service::state::{state_0, state_1, state_2};

use super::State;
use super::types::{Request, Response, WorkerReceiver, WorkerSender};

pub struct Worker {
    client:   Client,
    sender:   WorkerSender, 
    receiver: WorkerReceiver, 
}

impl Worker {
    pub fn new(
        client: Client,
        sender: WorkerSender, 
        receiver: WorkerReceiver
    ) -> Self {
        Self { client, sender, receiver }
    }

    pub fn run(self) {
        if let Err(e) = self.run_inner() {
            println!("[IntakeScales::Service::Worker] Error while processing: {}", e);
        }
    }

    fn run_inner(mut self) -> anyhow::Result<()> {
        loop {
            let request = self.receiver.recv()?;

            match request {
                Request::UpdateState(state, plate_count) => {
                    Self::update_state(&mut self, state, plate_count)
                }
                Request::Terminate => { return Ok(()) },
            }
        }
    }

    fn update_state(&mut self, state: State, plate_count: u32) {
        let result = match state {
            State::Zero           => state_0::get_next_state(&self.client),
            State::One(state_one) => state_1::get_next_state(&self.client, state_one),
            State::Two(state_two) => state_2::get_next_state(&self.client, state_two, plate_count),
        };

        let _ = match result {
            Ok(v)  => self.sender.send(Response::NextState(v)),
            Err(e) => self.sender.send(Response::Error(e)),
        };
    }
}