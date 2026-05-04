use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub timestamp: u64,
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
    pub weight_0: Option<i16>,
    pub weight_1: Option<i16>,
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