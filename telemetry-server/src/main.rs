use std::{os::unix::net::UnixListener, path::{Path, PathBuf}, thread, time::Duration};
use duckdb::Connection;

mod ingest;
mod query;

fn main() -> anyhow::Result<()> {
    let db_path: PathBuf = "/home/entity/qitech/prototype-ff01/testing/sandbox/data.db".into();
    let connection = init_db(&db_path)?;

    let socket_path = "/tmp/qitech_telemetry.sock";

    // remove stale socket file if it exists
    if let Err(e) = std::fs::remove_file(socket_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }

    let ingest_listener = UnixListener::bind(socket_path)?;

    let mut ingest_handle = spawn_ingest(ingest_listener, connection);
    let mut query_handle  = spawn_query(db_path.clone());

    loop {
        thread::sleep(Duration::from_secs(1));

        if ingest_handle.is_finished() {
            let connection = init_db(&db_path)?;
            let ingest_listener = UnixListener::bind(socket_path)?;

            eprintln!("Ingest died → restarting");
            ingest_handle = spawn_ingest(ingest_listener, connection);
        }

        if query_handle.is_finished() {
            eprintln!("Query died → restarting");
            query_handle = spawn_query(db_path.clone());
        }
    }
}

fn spawn_ingest(
    ingest_listener: UnixListener, 
    connection: Connection
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = ingest::run(ingest_listener, connection) {
            eprintln!("Ingest exited with error: {}", e);
        }
    })
}

fn spawn_query(db_path: PathBuf) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Err(e) = query::run(9000, db_path) {
            eprintln!("Query server exited with error: {}", e);
        }
    })
}

pub fn init_db<P: AsRef<Path>>(path: P) -> duckdb::Result<Connection> {
    let connection = Connection::open(path)?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS weights (
            timestamp TIMESTAMP_NS,
            order_id  UINTEGER,
            weight_0  SMALLINT,
            weight_1  SMALLINT
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS plates (
            timestamp TIMESTAMP_NS,
            order_id  UINTEGER,
            peak SMALLINT NOT NULL,
            real SMALLINT NOT NULL
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS orders (
            order_id     UINTEGER PRIMARY KEY,
            worker_id    UINTEGER,
            bounds       SMALLINT[4],

            started_at   TIMESTAMP_NS NOT NULL,
            completed_at TIMESTAMP_NS
        )",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            timestamp TIMESTAMP_NS NOT NULL,
            category  TINYINT NOT NULL,
            message   VARCHAR NOT NULL
        )",
        [],
    )?;

    Ok(connection)
}