use std::{fs::{self, File, OpenOptions}, io::{self, BufWriter, Seek, SeekFrom, Write}, path::PathBuf, time::{Duration, Instant}};

use chrono::{DateTime, Local, NaiveDate, TimeZone};
use crossbeam::channel::{Receiver, RecvTimeoutError};

use crate::{WalEntry, archive};

pub struct Config {
    pub dir_logs: PathBuf,
    pub dir_archive: PathBuf,
    pub flush_interval: Duration,
    pub initial_date: Option<DateTime<Local>>,
}

pub fn run(
    config: Config, 
    rx: Receiver<WalEntry>,
) -> io::Result<()> {
    let dir_days   = config.dir_archive.join("days");
    let dir_orders = config.dir_archive.join("orders");
    fs::create_dir_all(&dir_days)?;
    fs::create_dir_all(&dir_orders)?;

    let mut date_now = config.initial_date.unwrap_or(Local::now()).date_naive();
    let mut wal_file = open_wal_file(&config.dir_logs, date_now)?;
    let mut flush_prev_ts = Instant::now();

    loop {
        let result = rx.recv_timeout(config.flush_interval / 8);

        if let Err(RecvTimeoutError::Disconnected) = &result {
            // log as INFO
            println!("channel closed, shutting down journal worker");
            wal_file.flush()?;
            wal_file.get_ref().sync_all()?;
            return Ok(());
        }

        if let Ok(entry) = result {
            let entry_date = Local::timestamp_opt(
                &Local,
                entry.timestamp as i64 / 1_000_000, 
                0
            ).unwrap().date_naive();

            // rollover check
            if entry_date != date_now {
                wal_file.flush()?;
                wal_file.get_ref().sync_all()?;

                date_now = entry_date;
                wal_file = open_wal_file(&config.dir_logs, date_now)?;

                let config = archive::Config {
                    dir_logs: config.dir_logs.clone(),
                    dir_days: dir_days.clone(),
                    date_now,
                    thread_pool_size: 2,
                };

                archive::scan_and_process(config)?;
            }

            entry.write(&mut wal_file)?;
        }

        if flush_prev_ts.elapsed() >= config.flush_interval {
            wal_file.flush()?;
            wal_file.get_ref().sync_all()?;
            flush_prev_ts = Instant::now();
        }
    }
}

fn open_wal_file(
    logs_dir: &PathBuf,
    date: NaiveDate,
) -> io::Result<BufWriter<File>> {
    let file_name = format!("{}.wal", date.format("%Y%m%d"));
    let path = logs_dir.join(file_name);

    fs::create_dir_all(path.parent().unwrap())?;

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;

    let mut pos: u64 = 0;

    loop {
        let start = file.stream_position()?;

        match WalEntry::read(&mut file) {
            Ok(Some(_)) => {
                pos = file.stream_position()?;
            }
            Ok(None) => break,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                println!("WAL corruption detected at offset {}", start);
                file.set_len(start)?;
                file.sync_all()?;
                pos = start;
                break;
            }
            Err(e) => return Err(e),
        }
    }

    // move to end for appending
    file.seek(SeekFrom::Start(pos))?;

    Ok(BufWriter::new(file))
}