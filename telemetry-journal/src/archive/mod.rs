use std::{fs::{self, File}, io, path::PathBuf, sync::Arc, thread::{self, JoinHandle}};

use chrono::NaiveDate;
use crossbeam::queue::SegQueue;
use flate2::{Compression, write::GzEncoder};
use tar::Builder;

use telemetry_core::{Entry, Event};

mod weights;
pub use weights::WeightsArchive;

mod plates;
pub use plates::PlatesArchive;

mod order;
pub use order::OrdersArchive;

mod logs;
pub use logs::LogsArchive;

pub struct Config {
    pub dir_logs: PathBuf,
    pub dir_days: PathBuf,
    pub date_now: NaiveDate,
    pub thread_pool_size: u32,
}

pub fn scan_and_process(config: Config) -> io::Result<()> {
    let queue = scan(&config)?;
    process(&config.dir_days, queue, config.thread_pool_size)?;
    Ok(())
}

fn scan(config: &Config) -> io::Result<Arc<SegQueue<(NaiveDate, PathBuf)>>> {
    let queue = Arc::new(SegQueue::new());

    let entries = fs::read_dir(&config.dir_logs)?;
    for entry in entries.flatten() {
        let path = entry.path();

        let Some(filename_ext) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Some(name) = filename_ext.strip_suffix(".wal") else {
            fs::remove_file(path)?;
            continue;
        };

        let Some(date) = name_to_date(name) else {
            fs::remove_file(path)?;
            continue;
        };

        if date == config.date_now {
            // don't process current day, since were writing to it
            continue;
        }

        // check if archive has entry for this day
        if config.dir_days.join(name).exists() {
            // If it's older than 31 days, remove the WAL file
            // change logic to be if last elements and elements.len > 7
            if config.date_now - date >= chrono::Duration::days(31) {
                println!("File older than 31 days");
                fs::remove_file(&path)?;
            }

            // already processed, so skip
            println!("Skipping: {}", date);
            continue;
        }

        // put into queue for items that need to be processed
        queue.push((date, path));
    }

    Ok(queue)
}

fn process(
    dir_days: &PathBuf,
    queue: Arc<SegQueue<(NaiveDate, PathBuf)>>, 
    thread_pool_size: u32
) -> io::Result<()> {
    let mut handles: Vec<JoinHandle<io::Result<()>>> = Vec::new();

    for i in 0..thread_pool_size {
        let dir_days = dir_days.clone();
        let queue    = Arc::clone(&queue);

        handles.push(thread::spawn(move || worker(dir_days, queue, i)));
    }

    for h in handles {
        h.join().unwrap()?;
    }

    Ok(())
}

fn worker(
    dir_days: PathBuf,
    queue: Arc<SegQueue<(NaiveDate, PathBuf)>>,
    thread_id: u32,
) -> io::Result<()> {
    loop {
        match queue.pop() {
            Some((date, path)) => {
                let file = File::open(path)?;
                let tmp_dir = dir_days.join(format!("tmp_t{}", thread_id));
                let out_path = dir_days.join(format!("{}.tar.gz", date.format("%Y%m%d")));
                if let Err(e) = export_log(file, &tmp_dir, &out_path) {
                    eprintln!("error: {:?}", e);
                }
            }
            None => break,
        }
    }

    Ok(())
}

fn export_log(
    log_file: File,
    tmp_dir:  &PathBuf,
    out_path: &PathBuf,
) -> io::Result<()> {
    let (weights, plates, orders, logs) = replay_and_extract(log_file)?;

    // make sure tmp_dir is empty before we starting writing into it
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir)?;
    }

    // create tmp dir and export all necessary files into it
    fs::create_dir_all(&tmp_dir)?;
    weights.export(&tmp_dir.join("weights.parquet"))?;
    plates.export(&tmp_dir.join("plates.parquet"))?;
    orders.export(&tmp_dir.join("orders.parquet"))?;
    logs.export(&tmp_dir.join("logs.parquet"))?;

    // create archive file and write all files from tmp dir into it
    let archive_path = create_archive(&tmp_dir)?;

    // move archive from tmp to out path
    fs::rename(&archive_path, out_path)?;

    // clear tmp dir again
    fs::remove_dir_all(&tmp_dir)?;

    Ok(())
}

fn replay_and_extract(
    mut file: File
) -> io::Result<(WeightsArchive, PlatesArchive, OrdersArchive, LogsArchive)> {
    let mut weights = WeightsArchive::default();
    let mut plates  = PlatesArchive::default();
    let mut orders  = OrdersArchive::default();
    let mut logs    = LogsArchive::default();

    loop {
        let entry = match Entry::read(&mut file) {
            Ok(opt_v) => match opt_v {
                Some(v) => v,
                None => break,
            },
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    println!("Found corrupted log!");
                    break;
                }

                return Err(e);
            },
        };

        println!("Entry: {:?}", entry);

        let ts = entry.timestamp;

        use Event::*;
        match entry.event {
            Weight(event) => weights.push(ts, event),
            Plate(event)  => plates.push(ts, event),
            Order(event)  => orders.push(ts, event),
            Log(event)    => logs.push(ts, event),
        }
    }

    Ok((weights, plates, orders, logs))
}

fn create_archive(dir: &PathBuf) -> io::Result<PathBuf> {
    let archive_path = dir.join("archive.tar.gz");
    let tar_gz  = fs::File::create(&archive_path)?;
    let enc     = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    // 2. append everything in tmp_dir
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path  = entry.path();

        // skip the archive itself if rerun-safe
        if path == archive_path {
            continue;
        }

        tar.append_path_with_name(&path, path.file_name().unwrap())?;
    }

    tar.finish()?;
    Ok(archive_path)
}

fn name_to_date(name: &str) -> Option<NaiveDate> {
    if name.len() != 8 || !name.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    NaiveDate::parse_from_str(name, "%Y%m%d").ok()
}