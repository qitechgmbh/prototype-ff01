use std::{io::Write, path::PathBuf};

use chrono::{DateTime, Local};
use telemetry_core::{Entry, wal_path_from_date};
use tokio::{fs::File, io::{self, AsyncReadExt}};

pub struct LiveDataCache {
    pub data:  Vec<u8>,
    pub start: u64,
}

impl LiveDataCache {
    pub async fn new(dir_logs: &PathBuf) -> io::Result<Self> {
        let cutoff = Self::cutoff();

        let mut data   = vec![0u8; 400 * 1024 * 1024];
        let mut cursor = Cursor::new(&mut data);

        let today     = Local::now();
        let yesterday = today - chrono::Duration::days(1);

        // extract entries of the last 24 hours from .wal files
        extract_entries(dir_logs, yesterday, &mut cursor, cutoff).await?;
        extract_entries(dir_logs, today,     &mut cursor, cutoff).await?;

        Ok(Self { data, start: 0 })
    }

    pub fn extract_prev_24h<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.update_start()?;
        let start = self.start as usize;
        writer.write_all(&self.data[start..])?;
        Ok(())
    }

    pub fn append(&mut self, entry: Entry) -> io::Result<()> {
        let start    = self.start as usize;
        let capacity = self.data.len() as u64;

        let mut cursor = Cursor::new(&mut self.data[start..]);
        entry.write(&mut cursor);
        self.start += cursor.position();

        // Rotate buffer if 90% of capacity is reached
        if self.start >= capacity * 9 / 10 {
            self.rotate()?;
        }

        Ok(())
    }

    fn cutoff() -> u64 {
        (Utc::now().timestamp_micros() as u64) - MICROSECONDS_PER_DAY
    }

    fn rotate(&mut self) -> io::Result<()> {
        let capacity = self.data.len();

        let mut new_buf = vec![0u8; capacity];

        self.update_start()?;
        let start = self.start as usize;
        let end   = self.data.len() - start;

        new_buf[..end].copy_from_slice(&self.data[start..end]);

        self.data  = new_buf;
        self.start = end as u64;

        Ok(())
    }

    fn update_start(&mut self) -> io::Result<()> {
        use chrono::Utc;

        let cutoff = (Utc::now().timestamp_micros() - 24 * 60 * 60 * 1_000_000) as u64;

        let mut cursor = Cursor::new(&self.data[(self.start as usize)..]);
        let mut new_start = self.start;

        loop {
            let entry_start = cursor.position();

            let len = u16::from_be_bytes(len_buf) as usize;

            let entry = match Entry::read(&mut cursor) {
                Ok(Some(v)) => v,
                Ok(None) => break,
                Err(e) => return Err(e),
            };

            let entry_end = cursor.position();

            if entry.timestamp >= cutoff {
                new_start = entry_start;
                break;
            }
            new_start = entry_end;
        }

        self.start = new_start;

        Ok(())
    }
}

// utils
async fn extract_entries<W: Write>(
    dir_logs: &PathBuf, 
    date: DateTime<Local>,
    writer: &mut W,
    cutoff: u64
) -> io::Result<()> {
    let path = wal_path_from_date(dir_logs, date);
    let mut file = File::open(path).await?;

    loop {
        let mut len_buf = [0u8; 2];
        if file.read_exact(&mut len_buf).await.is_err() {
            break;
        }

        let len = u16::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        if file.read_exact(&mut buf).await.is_err() {
            break;
        }

        let entry = match Entry::decode(&buf) {
            Ok(v) => v,
            Err(e) => {
                // Assume corrupt entry is last entry so exit
                eprintln!("Failed to decode entry: {e}");
                break;
            },
        };

        if entry.timestamp >= cutoff {
            
            entry.encode(writer)?; // TODO: Remove this
        }
    }

    Ok(())
}