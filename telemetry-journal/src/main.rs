use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel::{Receiver, RecvError};
mod replay;

#[derive(Debug)]
pub enum WalEntry {
    Timeseries,
    Plate,
    Order,
    Log,
}

fn run(logs_dir: PathBuf, archive_days_dir: PathBuf, archive_orders_dir: PathBuf, rx: Receiver<(u64, WalEntry)>) {


    let mut last_ts: u64 = 0;



    loop {
        let Ok((ts, entry)) = rx.recv() else {
            println!("channel closed, shutting down journal worker");
            return;
        };



        //TODO: write data into wal
    }
}

pub fn main() {
    
}