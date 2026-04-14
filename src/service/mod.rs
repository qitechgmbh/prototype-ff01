use std::{collections::HashSet, time::Instant};

use anyhow::anyhow;
use beas_bsl::{Client, ClientConfig};

mod state;
use crossbeam::channel::bounded;
pub use state::State;

mod types;
use types::{Request, Response, Connection};
pub use types::Config;

mod worker;
use worker::Worker;

use crate::telemetry::{self, LogLevel};

#[derive(Debug)]
pub struct Service {
    config: Config,

    // state
    enabled: bool,
    state: State,
    state_mutation_counter: u64,
    // reconnect_attempts: u32,
    last_heartbeat_ts: Instant,
    last_reconnect_ts: Instant,
    last_send_ts:      Instant,
    connection: Option<Connection>,

    // completed
    completed_orders: HashSet<i32>
}

// public interface
impl Service {
    pub fn new(config: Config) -> Self {
        let now =  Instant::now();

        Self { 
            config,
            enabled: false, 
            state: State::Zero,
            state_mutation_counter: 0,
            // reconnect_attempts: 0,
            last_heartbeat_ts: now,
            last_reconnect_ts: now,
            last_send_ts:      now,
            connection: None,
            completed_orders: Default::default()
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, value: bool) {
        if self.enabled == value { return; }
        self.enabled = value;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mutation_counter(&self) -> u64 {
        self.state_mutation_counter
    }

    fn set_state(&mut self, state: State) {
        if self.state.index() == state.index() {
            return;
        }

        self.state_mutation_counter += 1;
        self.state = state;
    }

    fn get_or_create_connection(&mut self, now: Instant) -> anyhow::Result<Option<Connection>> {
        if self.connection.is_some() {
            return Ok(self.connection.take());
        }

        if self.enabled && now.duration_since(self.last_reconnect_ts) < self.config.timeout_reconnect {
            return Ok(None);
        }

        self.last_reconnect_ts = now;

        telemetry::log(
            LogLevel::Info, 
            format!("Establishing connection to Beas-Bsl...")
        );

        let connection = Self::create_connection(&self.config.config_path)?;

        telemetry::log(
            LogLevel::Info, 
            format!("Successfully established connection to Beas-Bsl")
        );

        self.set_state(State::Zero);
        Ok(Some(connection))
    }

    pub fn update(&mut self, now: Instant, plate_count: u32) -> anyhow::Result<bool> {
        if !self.enabled {
            self.set_state(State::Zero);
            self.connection = None;
            return Ok(false);
        }

        self.connection = self.get_or_create_connection(now)?;

        let Some(connection) = self.connection.as_mut() else {
            return Ok(false);
        };
        
        if connection.has_pending() {
            let Some(response) = connection.recv() else {
                if now.duration_since(self.last_heartbeat_ts) > self.config.timeout_heartbeat {
                    telemetry::log(
                        LogLevel::Warn, 
                        format!("Heartbeat timed out, dropping connection")
                    );

                    self.state = State::Zero;
                    self.connection = None;
                }

                return Ok(false);
            };

            self.last_heartbeat_ts = now;

            match response {
                Response::NextState(state) => {
                    if let State::One(data) = &state {
                        if self.completed_orders.contains(&data.entry.doc_entry) {
                            return Ok(false);
                        }
                    }

                    if let State::Two(data) = &state {
                        self.completed_orders.insert(data.state_one.entry.doc_entry);
                    }

                    self.state = state;
                },
                Response::Error(error) => {
                    return Err(anyhow!("Error in response: {:?}", error));
                },
            }
        } else {
            if now.duration_since(self.last_send_ts) < self.config.timeout_sending {
                return Ok(true);
            }

            connection.send(self.state.clone(), plate_count);
            self.last_send_ts = now;
        }

        return Ok(false);
    }
}

// utils
impl Service {
    fn create_connection(config_path: &str) -> anyhow::Result<Connection> {
        let client = Self::create_client(config_path)?;

        let (service_sender, worker_receiver)  = bounded::<Request>(512);
        let (worker_sender,  service_receiver) = bounded::<Response>(512);
        
        let connection = Connection::new(service_sender, service_receiver);
        let worker     = Worker::new(client, worker_sender, worker_receiver);
        
        let worker_handle = std::thread::spawn(move || Worker::run(worker));
        _ = worker_handle;

        Ok(connection)
    }

    pub fn create_client(config_path: &str) -> anyhow::Result<Client> {
        let config = ClientConfig::from_file(config_path)
            .map_err(|e| anyhow!("[Service_p10] Failed to read Config: {:?}", e))?;

        let client = Client::new(config)
            .map_err(|e| anyhow!("[Service_p10] Failed to create Client: {:?}", e))?;

        Ok(client)
    }
}