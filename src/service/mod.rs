use std::time::Instant;

use anyhow::anyhow;
use beas_bsl::{Client, ClientConfig};

mod state;
use crossbeam::channel::bounded;
pub use state::{State, StateOne, StateTwo};

mod types;
use types::{Request, Response, Connection};
pub use types::Config;

mod worker;
use worker::Worker;

#[derive(Debug)]
pub struct Service {
    config: Config,

    // state
    enabled: bool,
    state: State,
    reconnect_attempts: u32,
    last_heartbeat_ts: Instant,
    last_reconnect_ts: Instant,
    last_send_ts:      Instant,
    connection: Option<Connection>,
}

// public interface
impl Service {
    pub fn new(config: Config) -> Self {
        let now =  Instant::now();

        Self { 
            config,
            enabled: false, 
            state: State::Zero,
            reconnect_attempts: 0,
            last_heartbeat_ts: now,
            last_reconnect_ts: now,
            last_send_ts:      now,
            connection: None,
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
            self.state = State::Zero;
            self.connection = None;
            return Ok(());
        }

        let mut connection = if self.connection.is_none() {
            // self.reconnect_attempts += 1;

            // if self.reconnect_attempts > self.config.reconnect_attempts_max {
            //     panic!("Failed to connect, exceed reconnect_attempts_max!");
            // }

            if self.enabled && now.duration_since(self.last_reconnect_ts) > self.config.timeout_reconnect {
                self.last_reconnect_ts = now;
                println!("Event: Establishing connection");
                let connection = Self::create_connection(&self.config.config_path)?;
                self.connection = Some(connection);
                self.state = State::Zero;
            } else {
                return Ok(());
            }
            
            self.reconnect_attempts = 0;
            self.last_heartbeat_ts  = now;
            self.connection.as_mut().unwrap()
        } else {
            self.connection.as_mut().unwrap()
        };

        if connection.has_pending() {
            let Some(response) = connection.recv() else {
                if now.duration_since(self.last_heartbeat_ts) > self.config.timeout_heartbeat {
                    println!("Event: Heartbeat timed out, dropping connection");
                    self.state = State::Zero;
                    self.connection = None;
                }

                return Ok(());
            };

            match response {
                Response::NextState(state) => {
                    self.state = state;
                },
                Response::Error(error) => {
                    return Err(anyhow!("Error in response: {:?}", error));
                },
            }
        } else {
            if now.duration_since(self.last_send_ts) < self.config.timeout_sending {
                return Ok(());
            }

            connection.send(self.state.clone(), plate_count);
            self.last_send_ts = now;
        }

        return Ok(());
    }
}

// utils
impl Service {
    fn create_connection(config_path: &str) -> anyhow::Result<Connection> {
        let client = Self::create_client(config_path)?;

        let (service_sender, worker_receiver)  = bounded::<Request>(2);
        let (worker_sender,  service_receiver) = bounded::<Response>(2);
        
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