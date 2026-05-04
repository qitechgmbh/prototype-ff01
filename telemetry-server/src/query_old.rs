use std::{io, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{Router, extract::{Query, State}, response::IntoResponse, routing::get};
use duckdb::{types::Value};
use serde::Deserialize;

// $from=0, $to=100, $order=ORDER_ID
#[derive(Debug, Deserialize)]
struct TelemetryQuery {
    from:     Option<u64>,
    to:       Option<u64>,
    order_id: Option<u64>,
}

pub async fn run(query_port: u16, db_path: PathBuf) -> io::Result<()> {
    let state = Arc::new(db_path);

    

    let router = Router::new()
        .route("/api/telemetry/weights", get(weights))
        .route("/api/telemetry/plates",  get(plates))
        .route("/api/telemetry/orders",  get(orders))
        .route("/api/telemetry/logs",    get(logs))
        .with_state(state)
        ;

    let addr = SocketAddr::from(([0, 0, 0, 0], query_port));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

fn build_query(table_name: &str, params: TelemetryQuery) -> (String, Vec<Value>) {
    let mut sql = format!("SELECT * FROM {}", table_name);

    let mut conditions = Vec::new();
    let mut bind_values: Vec<Value> = Vec::new();

    // filters
    if let Some(order_id) = params.order_id {
        conditions.push("order_id = ?");
        bind_values.push(Value::from(order_id));
    }

    // WHERE
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // ORDER
    sql.push_str(" ORDER BY timestamp");

    // pagination
    match (params.from, params.to) {
        (Some(from), Some(to)) => {
            let limit = to.saturating_sub(from);
            sql.push_str(" LIMIT ? OFFSET ?");
            bind_values.push(Value::from(limit));
            bind_values.push(Value::from(from));
        }
        (Some(from), None) => {
            sql.push_str(" OFFSET ?");
            bind_values.push(Value::from(from));
        }
        (None, Some(to)) => {
            sql.push_str(" LIMIT ?");
            bind_values.push(Value::from(to));
        }
        (None, None) => {}
    }

    (sql, bind_values)
}

async fn weights(
    State(state): State<Arc<PathBuf>>,
    Query(params): Query<TelemetryQuery>
) -> impl IntoResponse {
    let db_path = state.clone();

    let result = tokio::task::spawn_blocking(move || {
        let connection = duckdb::Connection::open(&*db_path).unwrap();

        let (sql, values) = build_query("weights", params);

        let mut statement = connection.prepare(&sql).unwrap();

        // convert Vec<Value> → Vec<&dyn ToSql>
        let refs: Vec<&dyn duckdb::ToSql> =
            values.iter().map(|v| v as &dyn duckdb::ToSql).collect();

        let mut rows = statement.query(&refs[..]).unwrap();

        while let Some(row) = rows.next().unwrap() {
            let timestamp: i64 = row.get(0).unwrap();
            let order_id:  u32 = row.get(1).unwrap();
            let weight_0:  i16 = row.get(2).unwrap();
            let weight_1:  i16 = row.get(3).unwrap();

            println!(
                "timestamp={} order_id={} weight_0={} weight_1={}",
                timestamp, order_id, weight_0, weight_1
            );
        }

        // process rows...
        "ok"
    })
    .await;

    match result {
        Ok(v) => v.into_response(),
        Err(_) => "internal error".into_response(),
    }
}

async fn plates() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {

        // blocking DuckDB query
        "LIVE DATA BIATCH (from duckdb)"

        
    })
    .await;

    match result {
        Ok(v) => v,
        Err(_) => "internal error",
    }
}

async fn orders() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {

        // blocking DuckDB query
        "LIVE DATA BIATCH (from duckdb)"

        
    })
    .await;

    match result {
        Ok(v) => v,
        Err(_) => "internal error",
    }
}

async fn logs() -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {

        // blocking DuckDB query
        "LIVE DATA BIATCH (from duckdb)"

        
    })
    .await;

    match result {
        Ok(v) => v,
        Err(_) => "internal error",
    }
}