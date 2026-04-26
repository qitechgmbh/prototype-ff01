use std::{hash::Hasher, io::{self, Read, Write}};

use twox_hash::XxHash64;

use crate::{Fragment, MAGIC, VERSION};

#[derive(Debug, Clone, Copy, Default)]
pub struct ArchiveTier {
    pub capacity_desired: usize,
    pub capacity_max:     usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArchiveHeader {
    pub magic:          u32,
    pub version:        u32,
    pub page_size:      u32,
    pub fragment_count: u32,
}

impl ArchiveHeader {
    pub const BYTE_SIZE: usize = std::mem::size_of::<Self>();

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut u32_buf = [0u8; 4];
        let mut u64_buf = [0u8; 8];

        reader.read_exact(&mut u32_buf)?;
        let magic = u32::from_le_bytes(u32_buf);
            
        reader.read_exact(&mut u32_buf)?;
        let version = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let page_size = u32::from_le_bytes(u32_buf);

        reader.read_exact(&mut u32_buf)?;
        let fragment_count = u32::from_le_bytes(u32_buf);

        let zelf = ArchiveHeader {
            magic,
            version,
            page_size,
            fragment_count,
        };

        // checksum
        reader.read_exact(&mut u64_buf)?;
        let checksum_received = u64::from_le_bytes(u64_buf);

        if checksum_received != zelf.compute_checksum() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Checksum test failed"));
        }

        Ok(zelf)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let checksum = self.compute_checksum();

        writer.write_all(&self.magic.to_le_bytes())?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.page_size.to_le_bytes())?;
        writer.write_all(&self.fragment_count.to_le_bytes())?;
        writer.write_all(&checksum.to_le_bytes())?;
        Ok(())
    }

    fn compute_checksum(&self) -> u64 {
        let mut hasher = XxHash64::with_seed(0);
        hasher.write(&self.magic.to_le_bytes());
        hasher.write(&self.version.to_le_bytes());
        hasher.write(&self.page_size.to_le_bytes());
        hasher.write(&self.fragment_count.to_le_bytes());
        hasher.finish()
    }
}

#[derive(Debug, Clone)]
pub struct Archive<T: Fragment + Clone> {
    fragments: Vec<T>,
}

impl<T: Fragment + Clone> Archive<T> {
    pub fn new() -> Self {
        Self { fragments: Default::default() }
    }

    pub fn create(&mut self, fragment: T) -> Result<(), &'static str> {
        let (new_min, _) = fragment.header().range;

        if let Some(last) = self.fragments.last() {
            let (_, last_max) = last.header().range;

            if last_max > new_min {
                return Err("fragment out of order or overlapping");
            }
        }

        self.fragments.push(fragment);
        Ok(())
    }

    pub fn fragments(&self) -> &[T] {
        &self.fragments
    }

    pub fn encode<W: Write>(&self, writer: &mut W, page_size: u32) -> io::Result<()> {
        let mut size: u64 = 0;
        for fragment in &self.fragments {
            size += fragment.size();
        }

        let header = ArchiveHeader {
            magic:          MAGIC,
            version:        VERSION,
            fragment_count: self.fragments.len() as u32,
            data_size: size,
        };
        header.write(writer)?;

        for fragment in &self.fragments {
            fragment.encode(writer)?;
        }

        Ok(())
    }
}