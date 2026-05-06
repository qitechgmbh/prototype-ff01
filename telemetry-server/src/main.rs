use std::{env, sync::Arc, thread, time::Duration};

use crossbeam::channel::{Receiver, Sender, bounded};
use duckdb::Connection;

mod utils;
mod ingest;
mod live;
mod query;
mod recorder;
use recorder::Recorder;

type Payload         = Arc<Vec<u8>>;
type PayloadSender   = Sender<Payload>;
type PayloadReceiver = Receiver<Payload>;

pub struct Config {
    pub db_path:     String,
    pub socket_path: String,
    pub live_port:   u16,
    pub query_port:  u16,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let db_path     = env::var("DB_PATH")?;
        let socket_path = env::var("SOCKET_PATH")?;
        let live_port   = env::var("LIVE_PORT")?.parse::<u16>()?;
        let query_port  = env::var("QUERY_PORT")?.parse::<u16>()?;

        Ok(Self { 
            db_path, 
            socket_path, 
            live_port, 
            query_port 
        })
    }
}

fn main() -> anyhow::Result<()> {
    let config = Config::new()?;

    utils::init_db(&config.db_path)?;

    let (tx_recorder, rx_recorder) = bounded::<Payload>(4096);
    let (tx_live, rx_live)         = bounded::<Payload>(4096);

    let rx_recorder = Arc::new(rx_recorder);
    let rx_live     = Arc::new(rx_live);
    let subscribers = Arc::new([tx_recorder, tx_live]);

    let mut ingest_handle   = spawn_ingest(config.socket_path.clone(), subscribers.clone());
    let mut recorder_handle = spawn_recorder(config.db_path.clone(), rx_recorder.clone());
    let mut live_handle     = spawn_live(config.live_port, rx_live.clone());
    let mut query_handle    = spawn_query(config.db_path.clone(), config.query_port);

    loop {
        thread::sleep(Duration::from_secs(1));

        if ingest_handle.is_finished() {
            eprintln!("Ingest worker died -> restarting...");
            ingest_handle = spawn_ingest(config.socket_path.clone(), subscribers.clone());
        }

        if recorder_handle.is_finished() {
            eprintln!("Recorder worker died -> restarting...");
            std::process::exit(-2);
            recorder_handle = spawn_recorder(config.db_path.clone(), rx_recorder.clone());
        }

        if live_handle.is_finished() {
            eprintln!("Live worker died -> restarting...");
            live_handle = spawn_live(config.live_port, rx_live.clone());
        }

        if query_handle.is_finished() {
            eprintln!("Query worker died -> restarting...");
            query_handle = spawn_query(config.db_path.clone(), config.query_port);
        }
    }
}

fn spawn_ingest(
    socket_path: String,
    subscribers: Arc<[PayloadSender; 2]>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = ingest::run(socket_path, subscribers) {
            eprintln!("Ingest exited with error: {}", e);
        }
    })
}

fn spawn_recorder(
    db_path: String, 
    rx: Arc<PayloadReceiver>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        //TODO: don't panic, return error or smth idk
        let connection = Connection::open(db_path).expect("Failed to create connection");

        let recorder = Recorder::new(connection);
        if let Err(e) = recorder.run(rx) {
            eprintln!("Recorder exited with error: {}", e);
        }
    })
}

fn spawn_live(
    port: u16,
    rx: Arc<PayloadReceiver>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = live::run(port, rx) {
            eprintln!("Recorder exited with error: {}", e);
        }
    })
}

fn spawn_query(
    db_path: String,
    port: u16
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = query::run(port, db_path) {
            eprintln!("Query server exited with error: {}", e);
        }
    })
}