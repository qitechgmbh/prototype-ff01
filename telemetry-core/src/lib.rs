use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub timestamp: DateTime<Utc>,
    pub event:     Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Weight(WeightEvent),
    Plate(PlateEvent),
    Order(OrderEvent),
    Log(LogEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightEvent {
    pub order_id: Option<u32>,
    pub weight_0: Option<i16>,
    pub weight_1: Option<i16>,
}

impl WeightEvent {
    pub fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let mut flags: u8 = 0;
        if self.order_id.is_none() { flags |= 1 << 0; }
        if self.weight_0.is_none() { flags |= 1 << 1; }
        if self.weight_1.is_none() { flags |= 1 << 2; }
        buf[0] = flags;

        let mut i = 1;
        if let Some(v) = order_id {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        buf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateEvent {
    pub peak: i16,
    pub real: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    Started { order_id: u32, worker_id: Option<u32>, bounds: Option<WeightBounds> },
    Aborted,
    Completed { quantity_good: u32, quantity_scrap: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightBounds {
    min:     i16,
    max:     i16,
    desired: i16,
    trigger: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub category: LogCategory,
    pub message:  String,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogCategory {
    Debug,
    Info,
    Warn,
    Error,
}