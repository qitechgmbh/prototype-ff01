use std::{sync::Arc, thread, time::Duration};

use crossbeam::channel::{Receiver, Sender, bounded};

mod config;
use config::Config;

mod ingest;
mod live;
mod query;
mod recorder;
mod utils;
use recorder::Recorder;

type Payload = Arc<Vec<u8>>;
type PayloadSender = Sender<Payload>;
type PayloadReceiver = Receiver<Payload>;

const CHANNEL_CAPACITY:      usize = 4096;
const SUBSCRIBER_COUNT:      usize = 2;
const EVENT_SIZE_MAX:        usize = 256;
const EVENT_LEN_SIZE:        usize = 2;
const LIVE_CHANNEL_CAPACITY: usize = 128;
const RECORDER_CACHE_SIZE:   usize = 32;

fn main() -> anyhow::Result<()> {
    let config = Config::import()?;

    utils::init_db(&config.db_path)?;

    let (tx_recorder, rx_recorder) = bounded::<Payload>(CHANNEL_CAPACITY);
    let (tx_live, rx_live)         = bounded::<Payload>(CHANNEL_CAPACITY);

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
    subscribers: Arc<[PayloadSender; SUBSCRIBER_COUNT]>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = ingest::run(socket_path, subscribers) {
            eprintln!("Ingest service exited with error: {}", e);
        }
    })
}

fn spawn_recorder(db_path: String, rx: Arc<PayloadReceiver>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let recorder = match Recorder::new(db_path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Recorder service exited with error: {}", e);
                return;
            }
        };

        if let Err(e) = recorder.run(rx) {
            eprintln!("Recorder service exited with error: {}", e);
        }
    })
}

fn spawn_live(port: u16, rx: Arc<PayloadReceiver>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = live::run(port, rx) {
            eprintln!("Live service exited with error: {}", e);
        }
    })
}

fn spawn_query(db_path: String, port: u16) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = query::run(port, db_path) {
            eprintln!("Query service exited with error: {}", e);
        }
    })
}
