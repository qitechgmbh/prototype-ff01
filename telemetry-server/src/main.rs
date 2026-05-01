use axum::{
    routing::get,
    Router,
    extract::Path,
    response::IntoResponse,
};
use chrono::{DateTime, Local, Utc};
use telemetry_core::{Entry, wal_path_from_date};
use tokio::{io::{AsyncBufReadExt, BufReader}, net::TcpListener, sync::RwLock, task};
use std::{fs::File, io::{self, Cursor, Write}, net::SocketAddr, path::PathBuf, sync::Arc};

mod query;
mod cache;

const MICROSECONDS_PER_DAY: u64 = 86_400_000_000;

pub struct Config {
    pub dir_logs:    PathBuf,
    pub dir_archive: PathBuf,
}

pub struct LiveDataCache {
    pub data:  Vec<u8>,
    pub start: u64,
}

impl LiveDataCache {
    fn extract_entries<W: Write>(
        dir_logs: &PathBuf, 
        date: DateTime<Local>,
        writer: &mut W,
        cutoff: u64
    ) -> io::Result<()> {
        let path = wal_path_from_date(dir_logs, date);
        let mut file = File::open(path)?;

        loop {
            let entry = match Entry::read(&mut file) {
                Ok(Some(v)) => v,
                Ok(None) => break,
                Err(e) => return Err(e),
            };

            if entry.timestamp >= cutoff {
                entry.write(writer)?;
            }
        }

        Ok(())
    }

    pub fn new(dir_logs: &PathBuf) -> io::Result<Self> {
        let cutoff = Self::cutoff();

        let mut data   = vec![0u8; 400 * 1024 * 1024];
        let mut cursor = Cursor::new(&mut data);

        let today     = Local::now();
        let yesterday = today - chrono::Duration::days(1);

        // extract entries of the last 24 hours from .wal files
        Self::extract_entries(dir_logs, yesterday, &mut cursor, cutoff)?;
        Self::extract_entries(dir_logs, today,     &mut cursor, cutoff)?;

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


pub async fn run(config: Config) -> io::Result<()> {
    let cache = LiveDataCache::new(&config.dir_logs)?;
    let cache = Arc::new(RwLock::new(cache));



    Ok(())
}

#[derive(Clone)]
struct AppState {
    dir_logs: PathBuf,
    dir_archive: PathBuf,
    live_data_cache: Arc<RwLock<LiveDataCache>>
}

mod ingest;

#[tokio::main]
async fn main() -> io::Result<()> {
    use std::env::var;

    let dir_logs: PathBuf = var("DIR_LOGS")
        .expect("DIR_LOGS missing")
        .into();

    let dir_archive: PathBuf = var("DIR_ARCHIVE")
        .expect("DIR_ARCHIVE missing")
        .into();

    let ingest_port: u16 = var("INGEST_PORT").unwrap().parse().unwrap();
    let http_port:   u16 = var("HTTP_PORT").unwrap().parse().unwrap();

    let ingest_task = task::spawn(async move {
        run_ingest_server(ingest_port, ingest_state).await
    });







    let config = Config { dir_logs, dir_archive };
    let cache  = LiveDataCache::new(&config.dir_logs)?;
    let cache  = Arc::new(RwLock::new(cache));

    let app = Router::new()
        .route("/telemetry/live",                     get(live))
        .route("/telemetry/archive/days",             get(days))
        .route("/telemetry/archive/days/:date",       get(day_by_date))
        .route("/telemetry/archive/orders",           get(orders))
        .route("/telemetry/archive/orders/:order_id", get(orders));

    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn live() -> impl IntoResponse {
    "live telemetry"
}

async fn days() -> impl IntoResponse {
    "list of available days"
}

async fn day_by_date(Path(date): Path<String>) -> impl IntoResponse {
    format!("data for day {}", date)
}

async fn orders() -> impl IntoResponse {
    "orders archive"
}