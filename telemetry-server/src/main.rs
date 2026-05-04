use std::{os::unix::net::UnixListener, path::{Path, PathBuf}};
use duckdb::Connection;

mod ingest;
mod query_server;

fn main() -> anyhow::Result<()> {
    let db_path: PathBuf = "/home/entity/work/qitech/prototype-ff01/testing/sandbox/data.db".into();
    let connection = init_db(&db_path)?;

    let socket_path = "/tmp/qitech_telemetry.sock";

    // remove stale socket file if it exists
    if let Err(e) = std::fs::remove_file(socket_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e.into());
        }
    }

    let ingest_listener = UnixListener::bind(socket_path)?;

    // seperate thread handling ingestion of data
    _ = std::thread::spawn(|| {
        if let Err(e) = ingest::run(ingest_listener, connection) {
            eprintln!("Ingest exited with error: {}", e);
        }
    });

    // run query server
    query_server::run(9000, db_path)?;

    Ok(())
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
            category TINYINT NOT NULL,
            message  VARCHAR NOT NULL
        )",
        [],
    )?;

    Ok(connection)
}