use std::{io::Write, path::PathBuf};

use chrono::{DateTime, Local, Utc};
use telemetry_core::{Entry, EntryEncodeError, wal_path_from_date};
use tokio::{fs::File, io::{self, AsyncReadExt}};

use crate::MICROSECONDS_PER_DAY;

pub struct LiveDataCache {
    pub data:  Box<[u8; 512 * 1024 * 1024]>, // 512 MiB
    pub start: usize,
    pub end:   usize,
}

impl LiveDataCache {
    pub async fn new(dir_logs: &PathBuf) -> io::Result<Self> {
        let cutoff = Self::compute_cutoff();

        let mut instance = Self {
            data:  Box::new([0u8; 512 * 1024 * 1024]),
            start: 0,
            end:   0,
        };

        let today     = Local::now();
        let yesterday = today - chrono::Duration::days(1);

        instance.extract_from_log(dir_logs, yesterday, cutoff).await?;
        instance.extract_from_log(dir_logs, yesterday, cutoff).await?;

        Ok(instance)
    }

    pub async fn extract_prev_24h<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        self.update_start();
        let start = self.start as usize;
        writer.write_all(&self.data[start..])?;
        Ok(())
    }

    pub fn append(&mut self, entry: Entry) -> Result<(), EntryEncodeError> {
        let capacity = self.data.len();

        let data = entry.encode(self.data.as_mut_slice())?;
        self.end += data.len();

        // Rotate buffer if 90% of capacity is reached
        if self.start >= capacity * 9 / 10 {
            self.rotate();
        }

        Ok(())
    }

    fn compute_cutoff() -> u64 {
        (Utc::now().timestamp_micros() as u64) - MICROSECONDS_PER_DAY
    }

    fn rotate(&mut self) {
        let len = (self.end - self.start) as usize;

        let mut data = Box::new([0u8; 512 * 1024 * 1024]);

        let src = &self.data[self.start as usize..self.end as usize];
        data[..len].copy_from_slice(src);

        self.data  = data;
        self.start = 0;
        self.end   = len;
    }

    async fn extract_from_log(
        &mut self,
        dir_logs: &PathBuf, 
        date:     DateTime<Local>,
        cutoff:   u64
    ) -> io::Result<()> {
        let path = wal_path_from_date(dir_logs, date);
        let mut file = File::open(path).await?;

        loop {
            let mut len_buf = [0u8; 1];
            if file.read_exact(&mut len_buf).await.is_err() {
                break;
            }

            let len = u8::from_be_bytes(len_buf) as usize;

            let mut data_buf = vec![0u8; len];
            if file.read_exact(&mut data_buf).await.is_err() {
                break;
            }

            let entry = match Entry::decode(&data_buf) {
                Ok(v) => v,
                Err(e) => {
                    // Assume corrupt entry is last entry so exit
                    eprintln!("Failed to decode entry: {e}");
                    break;
                },
            };

            if entry.timestamp >= cutoff {
                self.append(entry);
            }
        }

        Ok(())
    }

    fn update_start(&mut self) {
        let cutoff = Self::compute_cutoff();

        let mut start = self.start;
        let mut data  = &self.data[start..];

        loop {
            let len   = data[0] as usize;
            let entry = Entry::decode(&data[1..1 + len])
                .expect("Corrupt entries should never pass .append()");

            if entry.timestamp < cutoff {
                start += 1 + len; // size + len
            }

            data = &data[1 + len..];
        }
    }
}
