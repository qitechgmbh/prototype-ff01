use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crc32fast::Hasher;

// machine emits live data
// telemetry archive

pub struct Wal {
    writer: BufWriter<File>,
}

impl Wal {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Wal {
            writer: BufWriter::new(file),
        })
    }

    pub fn append(&mut self, record: &[u8]) -> io::Result<()> {
        let len = record.len() as u32;

        let mut hasher = Hasher::new();
        hasher.update(record);
        let checksum = hasher.finalize();

        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&checksum.to_le_bytes())?;
        self.writer.write_all(record)?;

        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        Ok(())
    }
}