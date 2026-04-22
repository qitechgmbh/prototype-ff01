#[repr(u8)]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone)]
pub enum RecordRequest {
    Weight{ w0: f64, w1: f64 },
    Plate { peak: f64, avg: f64, exit: f64 },
    Bounds{ min: f64, max: f64, desired: f64, trigger: f64 },
    State { order_id: u32, state_id: u32},
    Order(RecordOrderRequest),
    Log(LogLevel, heapless::String<128>),
}

#[derive(Clone)]
pub struct RecordOrderRequest {
    pub order_id:         u32,
    pub personnel_id_buf: heapless::String<20>,
    pub quantity_scrap:   u16,
    pub quantity_good:    u16,
    pub time_start:       heapless::String<10>,
    pub time_end:         heapless::String<10>,
    pub duration:         u64,
}