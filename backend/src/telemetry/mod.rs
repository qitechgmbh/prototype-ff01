use std::sync::{Arc, OnceLock};

use arc_swap::ArcSwap;
use crossbeam::channel::Sender;

// TODO: REMOVE
mod test;

mod types;
use types::Shared;
pub use types::LogLevel;

mod functions;
pub use functions::record_weight;
pub use functions::record_plate;
pub use functions::record_bounds;
pub use functions::record_state;
#[allow(unused)]
pub use functions::record_order;
pub use functions::log;

mod binary;

mod writer;
use writer::Writer;
use writer::RecordRequest;
pub use writer::Config as WriterConfig;

mod server;
use server::Server;
pub use server::Config as ServerConfig;

mod archive_manager;
use archive_manager::ArchiveManager;
pub use archive_manager::Config as ArchiveManagerConfig;

mod segment;
// use segment::DataSegment;

static HANDLE: OnceLock<Sender<RecordRequest>> = OnceLock::new();

pub struct Config {
    pub writer:  writer::Config,
    pub server:  server::Config,
    pub archive: archive_manager::Config,
}

pub fn init(config: Config) {
    let (record_tx, record_rx)   = crossbeam::channel::unbounded();
    let (segment_tx, segment_rx) = crossbeam::channel::unbounded();

    HANDLE.set(record_tx).expect("Failed to init telemetry");

    let shared = Arc::new(Shared {
        // Careful: Default::default() leads to a stackoverflow here
        segment_snapshot: ArcSwap::from_pointee(None),
        segment_registry: Default::default(),
    });
    
    let writer = Writer::new(config.writer, record_rx, segment_tx);
    std::thread::spawn(move || writer.run());

    let archive_mgr = ArchiveManager::new(config.archive, shared.clone(), segment_rx)
        .expect("Failed to init archive manager");
    std::thread::spawn(move || archive_mgr.run());


    let server = Server::new(config.server, shared.clone())
        .expect("Failed to create server");
    std::thread::spawn(move || server.run());
}