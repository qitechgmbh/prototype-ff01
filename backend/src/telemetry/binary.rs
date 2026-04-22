use std::{fs::File, io::{self, Write}, path::PathBuf, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};

pub const BATCH_DURATION: Duration = Duration::from_secs(60 * 60 * 24);
pub const MAX_SAMPLES: usize = 12 * 60 * 6;

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchReadCursor {
    weights: u32,
}

#[derive(Debug)]
pub struct Batch {
    prev_id: Option<u64>,
    created: SystemTime,
    weights: WeightRecords,
    weights2: WeightRecords,
    // plates:  Vec<PlateRecord,  MAX_SAMPLES>,
    // bounds:  Vec<BoundsRecord, MAX_SAMPLES>,
    // states:  Vec<StateRecord,  MAX_SAMPLES>,
    // orders:  Vec<OrderRecord,  MAX_SAMPLES>,
    // logs:    Vec<LogRecord,    MAX_SAMPLES>,
}

impl Batch {
    pub fn new(now: SystemTime, prev_id: Option<u64>) -> Self {
        Self { prev_id: prev_id, created: now, weights: WeightRecords::default() }
    }

    pub fn created(&self) -> SystemTime {
        self.created
    }
    
    pub fn secs_since_ue(&self) -> u64 {
        self.created.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub fn append(&mut self, now: SystemTime, request: AppendRequest) {
        let elapsed = now.duration_since(self.created).unwrap().as_millis() as u16;

        match request {
            AppendRequest::WeightRecord(w0, w1) => {
                self.weights.dt.push(elapsed).expect("Should never be full");
                self.weights.w0.push((w0 * 10.0) as u16).expect("Should never be full");
                self.weights.w1.push((w1 * 10.0) as u16).expect("Should never be full");
            },
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // 1. created timestamp (you must define how you get unix time)
        let created_unix: u128 = 
            self.created.duration_since(UNIX_EPOCH).unwrap().as_millis();

        writer.write_all(&created_unix.to_le_bytes())?;

        // 2. number of samples
        let len = self.weights.dt.len() as u16;
        writer.write_all(&len.to_le_bytes())?;

        // 3. columnar data (raw memory copy)
        let dt = bytemuck::cast_slice(&self.weights.dt);
        let w0 = bytemuck::cast_slice(&self.weights.w0);
        let w1 = bytemuck::cast_slice(&self.weights.w1);

        writer.write_all(dt)?;
        writer.write_all(w0)?;
        writer.write_all(w1)?;

        Ok(())
    }

    pub fn append_weight(&mut self, now: SystemTime, w0: f64, w1: f64) {
        let elapsed = now.duration_since(self.created).unwrap().as_millis() as u16;
        self.weights.dt.push(elapsed).expect("Should never be full");
        self.weights.w0.push((w0 * 10.0) as u16).expect("Should never be full");
        self.weights.w1.push((w1 * 10.0) as u16).expect("Should never be full");
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeightRecords {
    pub dt: heapless::Vec<u16, MAX_SAMPLES>,
    pub w0: heapless::Vec<u16, MAX_SAMPLES>,
    pub w1: heapless::Vec<u16, MAX_SAMPLES>,
}

impl WeightRecords {
    pub fn append(&mut self, dt: SystemTime, w0: f64, w1: f64) {
        self.dt.push(elapsed).expect("Should never be full");
        self.w0.push((w0 * 10.0) as u16).expect("Should never be full");
        self.w1.push((w1 * 10.0) as u16).expect("Should never be full");
    }
}

#[derive(Clone)]
pub enum AppendRequest {
    WeightRecord(f64, f64),
}



#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PlateRecord {
    pub dt:   u16,
    pub peak: u16,
    pub avg:  u16,
    pub exit: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BoundsRecord {
    pub dt:      u16,
    pub min:     u16,
    pub max:     u16,
    pub desired: u16,
    pub trigger: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct StateRecord {
    pub order_id: u32,
    pub state_id: u32,
}

#[derive(Clone)]
pub struct OrderRecord {
    pub order_id:         u32,
    pub personnel_id_buf: heapless::String<20>,
    pub quantity_scrap:   u16,
    pub quantity_good:    u16,
    pub time_start:       heapless::String<10>,
    pub time_end:         heapless::String<10>,
    pub duration:         u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message_buf: [u8; 128],
    pub message_len: usize,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info  => write!(f, "INFO"),
            LogLevel::Warn  => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

fn app_dir() -> PathBuf {
    let base = dirs::home_dir().expect("no home directory");
    let path = base.join("telemetry");
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[cfg(test)]
mod tests {
    use crate::telemetry::archiving;

    use super::*;
    use std::{io::Cursor, path::PathBuf, thread::sleep};

    #[test]
    pub fn my_test() {
        let mut batch = Batch::new(SystemTime::now());

        std::thread::sleep(Duration::from_millis(1));
        batch.append(SystemTime::now(), AppendRequest::WeightRecord(0.0, 0.0));

        std::thread::sleep(Duration::from_millis(1));
        batch.append(SystemTime::now(), AppendRequest::WeightRecord(1.0, 3.0));

        // ---- write into in-memory buffer ----
        let mut buf = Cursor::new(Vec::<u8>::new());

        batch.write(&mut buf).unwrap();

        // ---- get raw bytes ----
        let bytes = buf.into_inner();

        // ---- print result ----
        println!("batch bytes: {:?}", bytes);

        archiving::save_batch(app_dir(), &batch).expect("Success");
    }
}