use std::{hash::Hasher, io::{self, Read, Write}};

use twox_hash::XxHash64;

use crate::{FragmentSchema, MAGIC, TableDyn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FragmentHeader {
    pub range: (u64, u64),
    pub size:  u64,
}

impl FragmentHeader {
    pub const BYTE_SIZE: usize = size_of::<u32>() + size_of::<u64>() * 4; 

    pub fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let checksum = self.compute_checksum();

        writer.write_all(&MAGIC.to_le_bytes())?;
        writer.write_all(&self.range.0.to_le_bytes())?;
        writer.write_all(&self.range.1.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;
        writer.write_all(&checksum.to_le_bytes())?;
        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut u32_buf = [0u8; 4];
        let mut u64_buf = [0u8; 8];

        // magic
        reader.read_exact(&mut u32_buf)?;
        let magic_received = u32::from_le_bytes(u32_buf);

        if magic_received != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Magic mismatch"));
        }

        // range.0
        reader.read_exact(&mut u64_buf)?;
        let from = u64::from_le_bytes(u64_buf);

        // range.1
        reader.read_exact(&mut u64_buf)?;
        let to = u64::from_le_bytes(u64_buf);

        // size
        reader.read_exact(&mut u64_buf)?;
        let size = u64::from_le_bytes(u64_buf);

        let zelf = FragmentHeader {
            range: (from, to),
            size,
        };

        // checksum
        reader.read_exact(&mut u64_buf)?;
        let checksum_received = u64::from_le_bytes(u64_buf);

        if checksum_received != zelf.compute_checksum() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Checksum test failed"));
        }

        Ok(zelf)
    }

    fn compute_checksum(&self) -> u64 {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(&MAGIC.to_le_bytes());
        hasher.write(&self.range.0.to_le_bytes());
        hasher.write(&self.range.1.to_le_bytes());
        hasher.write(&self.size.to_le_bytes());
        hasher.finish()
    }
}

pub trait Fragment where Self: Sized {
    const SCHEMA: FragmentSchema;

    fn header(&self) -> FragmentHeader;
    fn tables(&self) -> &[&dyn TableDyn];
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self>;
    fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}