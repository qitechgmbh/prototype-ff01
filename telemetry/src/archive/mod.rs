use std::hash::Hasher;

use twox_hash::XxHash64;

mod header;
pub use header::ArchiveHeader;

mod reader;
pub use reader::Reader;

mod writer;
pub use writer::Writer;

mod fragment;
pub use fragment::Fragment;
pub use fragment::Metadata as FragmentMetadata;
pub use fragment::Header as FragmentHeader;
pub use fragment::Body as FragmentBody;

mod manager;
pub use manager::Config as ManagerConfig;
pub use manager::ArchiveManager;
pub use manager::TierRegistry;

pub const MAGIC:   u64 = 0xB16B00B5;
pub const VERSION: u16 = 0000_1000; // v0.1

#[derive(Debug, Clone, Copy, Default)]
pub struct ArchiveTier {
    pub triggger: u16,
    pub capacity: u16,
}

fn compute_checksum(data: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(data);
    hasher.finish()
}