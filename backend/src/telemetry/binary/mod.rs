use std::{fmt::Debug, hash::Hasher, io::{self, Read, Write}};

use twox_hash::XxHash64;

mod header;
pub use header::ArchiveHeader;
pub use header::FragmentHeader;

mod reader; 
pub use reader::ArchiveReader;

mod writer; 
pub use writer::ArchiveWriter;

pub const MAGIC:   u64 = 0xDEADBEEF;
pub const VERSION: u16 = 0000_1000; // v0.1

pub trait Fragment where Self: Sized + Debug {
    fn range(&self) -> (u64, u64);
    fn encode_body<W: Write>(&self, writer: &mut W) -> io::Result<()>;
    fn decode<R: Read>(reader: &mut R, range: (u64, u64)) -> io::Result<Self>;
}

fn checksum(data: &[u8]) -> u64 {
    let mut hasher = XxHash64::with_seed(0);
    hasher.write(data);
    hasher.finish()
}