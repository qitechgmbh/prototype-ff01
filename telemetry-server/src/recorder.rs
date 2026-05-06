use std::{fmt::Debug, sync::Arc};
use chrono::{DateTime, Utc};
use duckdb::{Connection, params};

use telemetry_core::{Event, EventKind, LogEvent, OrderEvent, PlateEvent, WeightEvent};

use crate::PayloadReceiver;

#[derive(Debug)]
struct EventEntry<T: Debug> {
    pub datetime: DateTime<Utc>,
    pub event: T,
}

const CACHE_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Recorder {
    connection: Connection,
    weights:    heapless::Vec<EventEntry<WeightEvent>, CACHE_SIZE>,
    plates:     heapless::Vec<EventEntry<PlateEvent>,  CACHE_SIZE>,
    orders:     heapless::Vec<EventEntry<OrderEvent>,  CACHE_SIZE>,
    logs:       heapless::Vec<EventEntry<LogEvent>,    CACHE_SIZE>,
}

impl Recorder {
    pub fn new(connection: Connection) -> Self {
        Self { 
            connection, 
            weights: Default::default(),
            plates:  Default::default(),
            orders:  Default::default(),
            logs:    Default::default(),
        }
    }

    pub fn run(mut self, rx: Arc<PayloadReceiver>) -> anyhow::Result<()> {
        loop {
            let data = match rx.recv() {
                Ok(v) => v,
                Err(e) => {
                    // try to flush all before exiting
                    // _ = self.flush_weights();
                    // _ = self.flush_plates();
                    // _ = self.flush_orders();
                    // _ = self.flush_logs();
                    return Err(e.into());
                },
            };

            let event = Event::decode(&data)
                .expect("Ingest must validate data before sending");

            self.append_entry(event)?;
        }
    }

    fn append_entry(&mut self, event: Event) -> anyhow::Result<()> {
        match event.kind {
            EventKind::Weight(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if let Err(v) = self.weights.push(entry) {
                    self.flush_weights()?;
                    self.weights.push(v).expect("Should be empty");
                };
            },
            EventKind::Plate(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if let Err(v) = self.plates.push(entry) {
                    self.flush_plates()?;
                    self.plates.push(v).expect("Should be empty");
                };
            },
            EventKind::Order(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if let Err(v) = self.orders.push(entry) {
                    self.flush_orders()?;
                    self.orders.push(v).expect("Should be empty");
                };
            },
            EventKind::Log(kind) => {
                let entry = EventEntry {
                    datetime: event.datetime,
                    event:    kind,
                };

                if let Err(v) = self.logs.push(entry) {
                    self.flush_logs()?;
                    self.logs.push(v).expect("Should be empty");
                };
            },
        }

        Ok(())
    }

    fn flush_weights(&mut self) -> anyhow::Result<()> {
        if self.weights.is_empty() {
            return Ok(());
        }

        println!("[Recorder] Flushing {} weight events", self.weights.len());

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO weights VALUES (?, ?, ?, ?)")?;

        for entry in self.weights.drain(..) {
            let datetime = format!("{}", entry.datetime.format("%Y-%m-%d %H:%M:%S%.f"));

            stmt.execute(
                params![
                    datetime,
                    entry.event.order_id,  
                    entry.event.weight_0, 
                    entry.event.weight_1
            ])?;
        }

        tx.commit()?;

        Ok(())
    }

    fn flush_plates(&mut self) -> anyhow::Result<()> {
        if self.plates.is_empty() {
            return Ok(());
        }

        println!("[Recorder] Flushing {} plate events", self.plates.len());

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO plates VALUES (?, ?, ?, ?)")?;

        for entry in self.plates.drain(..) {
            let datetime = format!("{}", entry.datetime.format("%Y-%m-%d %H:%M:%S%.f"));

            stmt.execute(
                params![
                    datetime,
                    entry.event.order_id,
                    entry.event.peak,
                    entry.event.real,
            ])?;
        }
        tx.commit()?;

        Ok(())
    }

    fn flush_orders(&mut self) -> anyhow::Result<()> {
        if self.orders.is_empty() {
            return Ok(());
        }

        println!("[Recorder] Flushing {} order events", self.orders.len());

        let tx = self.connection.transaction()?;

        let mut stmt_started = tx.prepare(r#"
            INSERT INTO orders 
            VALUES (?, ?, ?, [?, ?, ?, ?], ?, ?, ?, ?)
        "#)?;

        let mut stmt_aborted = tx.prepare(r#"
            UPDATE orders
            SET status = 'aborted',
                closed_at = ?
            WHERE order_id = ?
        "#)?;

        let mut stmt_completed = tx.prepare(r#"
            UPDATE orders
            SET status = 'completed',
                quantity_good = ?,
                quantity_scrap = ?,
                closed_at = ?
            WHERE order_id = ?
        "#)?;

        for entry in self.orders.drain(..) {
            let datetime = format!("{}", entry.datetime.format("%Y-%m-%d %H:%M:%S%.f"));

            match entry.event {
                OrderEvent::Started { order_id, worker_id, bounds } => {
                    stmt_started.execute(
                        params![
                            order_id,
                            worker_id,
                            "started",
                            bounds.as_ref().map(|b| b.min),
                            bounds.as_ref().map(|b| b.max),
                            bounds.as_ref().map(|b| b.desired),
                            bounds.as_ref().map(|b| b.trigger),
                            0, 
                            0, 
                            datetime,
                            None::<String>
                    ])?;
                }

                OrderEvent::Completed { order_id, quantity_good, quantity_scrap } => {
                    stmt_completed.execute(params![
                        quantity_good,
                        quantity_scrap,
                        datetime,
                        order_id,
                    ])?;
                }

                OrderEvent::Aborted { order_id } => {
                    stmt_aborted.execute(params![
                        datetime,
                        order_id,
                    ])?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn flush_logs(&mut self) -> anyhow::Result<()> {
        if self.logs.is_empty() {
            return Ok(());
        }

        println!("[Recorder] Flushing {} log events", self.logs.len());

        let tx = self.connection.transaction()?;
        let mut stmt = tx.prepare("INSERT INTO logs VALUES (?, ?, ?)")?;

        for entry in self.logs.drain(..) {
            let datetime = format!("{}", entry.datetime.format("%Y-%m-%d %H:%M:%S%.f"));

            stmt.execute(
                params![
                    datetime,
                    entry.event.category as u8,
                    entry.event.message.as_bytes(),
            ])?;
        }
        tx.commit()?;

        Ok(())
    }
}