use std::{io::Read, os::unix::net::UnixListener};

use duckdb::{Appender, Connection, params};
use telemetry_core::{Entry, Event, LogEvent, OrderEvent, PlateEvent, WeightEvent};

struct State<'a> {
    // appenders
    pub weights: Appender<'a>,
    pub plates:  Appender<'a>,

    #[allow(unused)]
    pub orders:  Appender<'a>,
    pub logs:    Appender<'a>,

    // buffers
    pub weight_buf: Vec<(u64, WeightEvent)>,
    pub plate_buf:  Vec<(u64, PlateEvent)>,
    pub order_buf:  Vec<(u64, OrderEvent)>,
    pub log_buf:    Vec<(u64, LogEvent)>,
}

impl<'a> State<'a> {
    pub fn new(connection: &'a duckdb::Connection) -> anyhow::Result<Self> {
        Ok(Self {
            weights: connection.appender("weights")?,
            plates:  connection.appender("plates")?,
            orders:  connection.appender("orders")?,
            logs:    connection.appender("logs")?,

            weight_buf: Vec::with_capacity(32),
            plate_buf:  Vec::with_capacity(32),
            order_buf:  Vec::with_capacity(32),
            log_buf:    Vec::with_capacity(32),
        })
    }

    pub fn append(&mut self, entry: &Entry) {
        println!("Appending: {:?}", entry);

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
                self.log_buf.push((entry.timestamp, event.clone()));
            }
        }
    }

    pub fn is_full(&self) -> bool {
        self.weight_buf.len() > 32
        || self.plate_buf.len() > 32
        || self.order_buf.len() > 32
        || self.log_buf.len() > 32
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        for (ts, e) in self.weight_buf.drain(..) {
            self.weights.append_row(params![ts, e.weight_0, e.weight_1])?;
        }

        for (ts, e) in self.plate_buf.drain(..) {
            self.plates.append_row(params![ts, e.peak, e.real])?;
        }

        for (ts, e) in self.order_buf.drain(..) {
            let _ = (ts, e);
        }

        for (ts, e) in self.log_buf.drain(..) {
            self.logs.append_row(params![ts, e.category as u8, e.message])?;
        }

        Ok(())
    }
}

pub fn run(listener: UnixListener, connection: Connection) -> anyhow::Result<()> {
    loop {
        let (mut stream, _) = listener.accept()?;

        let mut state = State::new(&connection)?;

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
            stream.read_exact(&mut buf)?;

            let entry: Entry = postcard::from_bytes(&buf)?;
            state.append(&entry);

            if state.is_full() {
                state.flush()?;
            }
        }
    }
}