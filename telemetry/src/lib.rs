mod format;
pub use format::FragmentSchema;
pub use format::TableSchema;
pub use format::ColumnSchema;

pub trait Fragment<'a> {
    const SCHEMA: FragmentSchema;

    fn tables(&self) -> &[&'a dyn TableDyn];
}

pub trait Table {
    type Item;
    const SCHEMA: TableSchema;

    fn append(&mut self, ts: u64, item: Self::Item);
}

pub trait TableDyn {
    fn schema(&self) -> &'static TableSchema;
    fn columns(&self) -> &[Column];
}

#[derive(Debug, Clone)]
pub enum Column<'a> {
    Unsigned8(&'a [u8]),
    Unsigned16(&'a [u16]),
    Unsigned32(&'a [u32]),
    Unsigned64(&'a [u64]),

    Signed8(&'a [i8]),
    Signed16(&'a [i16]),
    Signed32(&'a [i32]),
    Signed64(&'a [i64]),

    Float32(&'a [f32]),
    Float64(&'a [f64]),

    String(&'a [String]),
}

/*
use std::{io, sync::Arc, thread};

use arc_swap::ArcSwap;
use crossbeam::channel::Sender;

mod format;
mod types;
// mod stitcher;

mod recorder;
use recorder::Recorder;
pub use recorder::Config as RecorderConfig;

pub mod archive;
use archive::{ArchiveManager, FragmentBody, TierRegistry};
pub use archive::ManagerConfig as ArchiveManagerConfig;

use crate::archive::Fragment;

pub struct Config {
    pub recorder: RecorderConfig,
    pub archive:  ArchiveManagerConfig,
}

pub struct TelemetrySystem<const N: usize, F: FragmentBody<N>> {
    pub record_tx: Sender<F::Record>,
    shared: Arc<Shared<N, F>>
}

impl<const N: usize, F: FragmentBody<N>> TelemetrySystem<N, F> {
    pub fn start(config: Config) -> io::Result<Self> {
        let (record_tx, record_rx)   = crossbeam::channel::unbounded();
        let (segment_tx, segment_rx) = crossbeam::channel::unbounded();

        let shared = Arc::new(Shared {
            fragment_snapshot: ArcSwap::from_pointee(None),
            tier_registry: Default::default(),
        });

        let recorder = Recorder::new(config.recorder, record_rx, segment_tx);
        thread::spawn(move || recorder.run());

        let archive_manager = ArchiveManager::new(config.archive, shared.clone(), segment_rx)
            .expect("Failed to init archive manager");
        thread::spawn(move || archive_manager.run());

        Ok(Self { record_tx, shared })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Shared<const N: usize, Body: FragmentBody<N>> {
    pub fragment_snapshot: ArcSwap<Option<Fragment<N, Body>>>,
    pub tier_registry:     ArcSwap<TierRegistry>
}

    */