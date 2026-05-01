use std::{io, net::SocketAddr};

use axum::{Router, extract::Path, response::IntoResponse, routing::get};

use crate::AppState;

pub async fn run(state: AppState, query_port: u16) -> io::Result<()> {
    _ = state;
    
    let app = Router::new()
        .route("/telemetry/live",                     get(live))
        .route("/telemetry/archive/days",             get(days))
        .route("/telemetry/archive/days/:date",       get(day_by_date))
        .route("/telemetry/archive/orders",           get(orders))
        .route("/telemetry/archive/orders/:order_id", get(orders));

    let addr = SocketAddr::from(([0, 0, 0, 0], query_port));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
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