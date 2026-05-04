use std::{io, net::SocketAddr};

use axum::{Router, response::IntoResponse, routing::get};

pub async fn run(query_port: u16) -> io::Result<()> {
    let app = Router::new()
        .route("/api/telemetry/live",                     get(live))
        .route("/api/telemetry/archive/days",             get(days))
        // .route("/api/telemetry/archive/days/:date",       get(day_by_date))
        .route("/api/telemetry/archive/orders",           get(orders))
        .route("/api/telemetry/archive/orders/:order_id", get(orders))
        ;

    let addr = SocketAddr::from(([0, 0, 0, 0], query_port));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn live() -> impl IntoResponse {
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

async fn days() -> impl IntoResponse {
    "list of available days"
}

async fn orders() -> impl IntoResponse {
    "list of available orders"
}