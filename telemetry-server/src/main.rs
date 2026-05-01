use std::{io::{self}, net::SocketAddr, path::PathBuf, sync::Arc};

use tokio::{sync::RwLock, task};
use axum::{
    Router, extract::Path, http, response::IntoResponse, routing::get
};


mod query;

mod cache;
use cache::LiveDataCache;

const MICROSECONDS_PER_DAY: u64 = 86_400_000_000;

pub struct Config {
    pub dir_logs:    PathBuf,
    pub dir_archive: PathBuf,
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

    let live_data_cache = LiveDataCache::new(&dir_logs).await?;
    let live_data_cache = Arc::new(RwLock::new(live_data_cache));

    let app_state = AppState { dir_logs, dir_archive, live_data_cache };

    task::spawn({
        let state = app_state.clone();
        async move {
            ingest::run(ingest_port, state).await;
        }
    });

    _ = task::spawn({
        let state =  app_state.clone();
        async move {
            query::run(state, http_port).await;
        }
    });

    Ok(())
}

async fn run(
    dir_logs: PathBuf, 
    dir_archive: PathBuf, 
    ingest_port: u16, 
    query_port: u16
) -> io::Result<()> {
    let live_data_cache = LiveDataCache::new(&dir_logs).await?;
    let live_data_cache = Arc::new(RwLock::new(live_data_cache));

    let app_state = AppState { dir_logs, dir_archive, live_data_cache };

    _ = task::spawn(async move {
        if let Err(e) = ingest::run(ingest_port, app_state.clone()).await {
            eprintln!("Error in Ingest Task: {}", e);
        }
    });

    let app = Router::new()
        .route("/telemetry/live",                     get(live))
        .route("/telemetry/archive/days",             get(days))
        .route("/telemetry/archive/days/:date",       get(day_by_date))
        .route("/telemetry/archive/orders",           get(orders))
        .route("/telemetry/archive/orders/:order_id", get(orders));

    let addr = SocketAddr::from(([0, 0, 0, 0], query_port));
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