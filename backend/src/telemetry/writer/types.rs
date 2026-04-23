use crate::telemetry::types::LogLevel;

#[allow(unused)]
#[derive(Clone)]
pub enum RecordRequest {
    Weight{ w0: i16, w1: i16 },
    Plate { peak: i16, avg: i16 },
    Bounds{ min: i16, max: i16, desired: i16, trigger: i16 },
    State { order_id: u32, state_id: u32 },
    Order(OrderRecord),
    Log(LogLevel, String),
}

#[derive(Clone)]
pub struct OrderRecord {
    pub order_id:         u32,
    pub personnel_id: heapless::String<20>,
    pub quantity_scrap:   u16,
    pub quantity_good:    u16,
    pub time_start:       heapless::String<10>,
    pub time_end:         heapless::String<10>,
    pub duration:         u32,
}