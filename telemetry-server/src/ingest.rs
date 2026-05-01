use std::{io::{self}, net::SocketAddr};
use telemetry_core::Entry;
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}};

use crate::AppState;

async fn run(port: u16, state: AppState) -> io::Result<()> {
    let address  = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(address).await?;

    println!("ingestion listening on {}", address);

    loop {
        let (socket, _) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(worker(state, socket));
    }
}

async fn worker(state: AppState, mut socket: TcpStream) {
    loop {
        let mut len_buf = [0u8; 2];
        if socket.read_exact(&mut len_buf).await.is_err() {
            break;
        }

        let len = u16::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; len];
        if socket.read_exact(&mut buf).await.is_err() {
            break;
        }

        match Entry::decode(&buf) {
            Ok(entry) => {
                let mut cache = state.live_data_cache.write().await;
                cache.append(entry) .expect("TODO: REMOVE THIS");
            },
            Err(e) => {
                eprintln!("failed to decode entry: {e}");
            },
        }
    }
}