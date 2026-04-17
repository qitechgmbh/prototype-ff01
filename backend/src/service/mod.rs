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

use crate::telemetry::{self, LogLevel, ServiceStateRecord};

#[derive(Debug)]
pub struct Service {
    config: Config,

    // state
    enabled: bool,
    state:   State,
    last_reconnect_ts: Instant,
    last_send_ts:      Instant,
    last_recv_ts:      Instant,
    connection:        Option<Connection>,
    completed_orders:  HashSet<i32>
}

// public interface
impl Service {
    pub fn new(config: Config) -> Self {
        let now =  Instant::now();

        Self { 
            config,
            enabled: false, 
            state: State::Zero,
            last_reconnect_ts: now,
            last_send_ts:      now,
            last_recv_ts:      now,
            connection: None,
            completed_orders: HashSet::new(),
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


    pub fn update(&mut self, now: Instant, plate_count: u32) -> anyhow::Result<()> {
        if !self.enabled {
            self.set_state(State::Zero);
            self.connection = None;
            return Ok(());
        }

        let Some(mut connection) = self.get_or_create_connection(now)? else {
            return Ok(());
        };
        
        let result = if connection.has_pending() {
            self.send_next(now, &mut connection)
        } else {
            self.recv_and_process(now, &mut connection, plate_count);
            Ok(())
        };

        self.connection = Some(connection);
        result
    }
}

// utils
impl Service {
    fn create_connection(config_path: &str) -> anyhow::Result<Connection> {
        let client = Self::create_client(config_path)?;

        let (service_sender, worker_receiver)  = bounded::<Request>(4);
        let (worker_sender,  service_receiver) = bounded::<Response>(4);
        
        let connection = Connection::new(service_sender, service_receiver);
        let worker     = Worker::new(client, worker_sender, worker_receiver);
        
        let worker_handle = std::thread::spawn(move || Worker::run(worker));
        _ = worker_handle;

        Ok(connection)
    }

    fn create_client(config_path: &str) -> anyhow::Result<Client> {
        let config = ClientConfig::from_file(config_path)
            .map_err(|e| anyhow!("[Service_p10] Failed to read Config: {:?}", e))?;

        let client = Client::new(config)
            .map_err(|e| anyhow!("[Service_p10] Failed to create Client: {:?}", e))?;

        Ok(client)
    }

    fn set_state(&mut self, state: State) {
        if self.state.index() == state.index() {
            return;
        }

        let order_id = match &state {
            State::Zero => 0,
            State::One(state) => state.entry.doc_entry,
            State::Two(state) => state.state_one.entry.doc_entry,
        };

        telemetry::record_state(ServiceStateRecord {
            state_id: state.index(),
            order_id: order_id,
        });

        self.state = state;
    }

    fn get_or_create_connection(&mut self, now: Instant) -> anyhow::Result<Option<Connection>> {
        if self.connection.is_some() {
            return Ok(self.connection.take());
        }

        if self.enabled && now.duration_since(self.last_reconnect_ts) < self.config.reconnect_delay {
            return Ok(None);
        }

        self.last_reconnect_ts = now;

        telemetry::log(
            LogLevel::Info, 
            format!("Establishing connection to Beas-Bsl...")
        );

        let connection = Self::create_connection(&self.config.config_path)?;
        self.last_send_ts = now;

        telemetry::log(
            LogLevel::Info, 
            format!("Established connection to Beas-Bsl successfully!")
        );

        self.set_state(State::Zero);
        Ok(Some(connection))
    }

    fn send_next(&mut self, now: Instant, connection: &mut Connection) -> anyhow::Result<()> {
        let Some(response) = connection.recv() else {
            if now.duration_since(self.last_recv_ts) > self.config.timeout_duration {
                telemetry::log(
                    LogLevel::Warn, 
                    format!(
                        "Timeout: Received no response after: {} seconds... Dropping connection", 
                        self.config.timeout_duration.as_secs_f64()
                    )
                );

                self.set_state(State::Zero);
                self.connection = None;
            }

            return Ok(());
        };

        self.last_recv_ts = now;

        match response {
            Response::NextState(state) => {
                if let State::One(data) = &state {
                    if self.completed_orders.contains(&data.entry.doc_entry) {
                        return Ok(());
                    }
                }

                if let State::Two(data) = &state {
                    self.completed_orders.insert(data.state_one.entry.doc_entry);
                }

                self.set_state(state);
                Ok(())
            },
            Response::Error(error) => {
                return Err(anyhow!("Error in response: {:?}", error));
            },
        }
    }

    fn recv_and_process(&mut self, now: Instant, connection: &mut Connection, plate_count: u32) {
        if now.duration_since(self.last_send_ts) < self.config.send_delay {
            return;
        }

        connection.send(self.state.clone(), plate_count);
        self.last_send_ts = now;
    }
}