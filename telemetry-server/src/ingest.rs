use std::{io::Read, os::unix::net::UnixListener};

use chrono::{DateTime, Utc};
use duckdb::{Connection, params};
use telemetry_core::{Entry, Event, LogEvent, OrderEvent, PlateEvent, WeightEvent};

struct State {
    pub connection: duckdb::Connection,

    pub weight_buf: Vec<(DateTime<Utc>, WeightEvent)>,
    pub plate_buf:  Vec<(DateTime<Utc>, PlateEvent)>,
    pub order_buf:  Vec<(DateTime<Utc>, OrderEvent)>,
    pub logs_buf:    Vec<(DateTime<Utc>, LogEvent)>,
}

impl State {
    pub fn new(connection: duckdb::Connection) -> anyhow::Result<Self> {
        Ok(Self {
            connection,
            weight_buf: Vec::with_capacity(32),
            plate_buf:  Vec::with_capacity(32),
            order_buf:  Vec::with_capacity(32),
            logs_buf:    Vec::with_capacity(32),
        })
    }

    pub fn append(&mut self, entry: &Entry) {
        match &entry.event {
            Event::Weight(event) => {
                self.weight_buf.push((entry.timestamp, event.clone()));
            }
            Event::Plate(event) => {
                self.plate_buf.push((entry.timestamp, event.clone()));
            }
            Event::Order(event) => {
                self.order_buf.push((entry.timestamp, event.clone()));
            }
            Event::Log(event) => {
                self.logs_buf.push((entry.timestamp, event.clone()));
            }
        }
    }

    pub fn is_full(&self) -> bool {
        self.weight_buf.len() > 32
        || self.plate_buf.len() > 32
        || self.order_buf.len() > 32
        || self.logs_buf.len() > 32
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO weights VALUES (?, ?, ?, ?)")?;

        for (ts, e) in self.weight_buf.drain(..) {
            let datetime = format!("{}", ts.format("%Y-%m-%d %H:%M:%S%.f"));
            stmt.execute(
                params![
                    datetime,
                    e.order_id,  
                    e.weight_0, 
                    e.weight_1
            ])?;
        }
        tx.commit()?;

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO plates VALUES (?, ?, ?)")?;
        for (ts, e) in self.plate_buf.drain(..) {
            let datetime = format!("{}", ts.format("%Y-%m-%d %H:%M:%S%.f"));
            stmt.execute(
                params![
                    datetime,
                    e.peak,
                    e.real,
            ])?;
        }
        tx.commit()?;

        // logs
        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO logs VALUES (?, ?, ?)")?;
        for (ts, e) in self.logs_buf.drain(..) {
            let datetime = format!("{}", ts.format("%Y-%m-%d %H:%M:%S%.f"));
            stmt.execute(
                params![
                    datetime,
                    e.category as u8,
                    e.message,
            ])?;
        }
        tx.commit()?;

        Ok(())
    }
}

pub fn run(listener: UnixListener, connection: Connection) -> anyhow::Result<()> {
    let mut state = State::new(connection)?;

    loop {
        let (mut stream, _) = listener.accept()?;

        loop {
            let mut len_buf = [0u8; 2];

            if let Err(e) = stream.read_exact(&mut len_buf) {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    state.flush()?;
                    break;
                }
                return Err(e.into());
            }

            let len = u16::from_le_bytes(len_buf) as usize;

            let mut buf = vec![0u8; len];
            if let Err(e) = stream.read_exact(&mut buf) {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    state.flush()?;
                    break;
                }
                return Err(e.into());
            }

            let entry: Entry = match postcard::from_bytes(&buf) {
                Ok(v) => v,
                Err(e) => {
                    println!("e: {}", e);
                    return Err(e.into());
                },
            };

            state.append(&entry);

            if state.is_full() {
                state.flush()?;
            }
        }
    }
}