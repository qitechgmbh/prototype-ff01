use std::fs::File;

use heapless::Vec;

const _: () = assert!(std::mem::size_of::<WeightRecord>() == 6);

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct WeightRecord {
    pub weight_0:     u16,
    pub weight_1:     u16,
    pub weight_total: u16, // 0.0..600.0 -> || 0..6000
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct PlateRecord {
    pub peak: u16,
    pub drop: u16,
    pub exit: u16,
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct BoundsRecord {
    pub min:     u16,
    pub max:     u16,
    pub desired: u16,
    pub trigger: u16,
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct StateRecord {
    pub order_id: u32,
    pub state_id: u32,
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct OrderRecord {
    pub order_id:         u32,
    pub personnel_id_buf: [u8; 20],
    pub personnel_id_len: u8,
    pub quantity_scrap:   u16,
    pub quantity_good:    u16,
    pub time_start:       u64,
    pub time_end:         u64,
    pub duration:         u64, // seconds
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message_buf: [u8; 96],
    pub message_len: usize,
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn encode_blob() {

}

pub const SAMPLE_RATE: f64 = 12.0;

const fn compute_limit(samples_per_minute_max: f64) -> usize {
    (samples_per_minute_max * 60.0 * 24.0 * 7.0) as usize
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TelemetryBlob {
    pub weights_buf: [WeightRecord; compute_limit(SAMPLE_RATE * 60.0)],
    pub weights_len: u32,
    pub plates_buf:  [PlateRecord; compute_limit(3.0 * 60.0)],
    pub plates_len:  u32,
    pub bounds_buf:  [BoundsRecord; compute_limit(1.0)],
    pub bounds_len:  u32,
    pub states_buf:  [StateRecord; compute_limit(1.0)],
    pub states_len:  u32,
    pub orders_buf:  [OrderRecord; compute_limit(1.0)],
    pub orders_len:  u32,
    pub logs_buf:    [LogRecord; compute_limit(60.0)],
    pub logs_len:    u32,
}

use bytemuck::cast_slice;

pub const WEIGHTS_LEN_MAX: usize = compute_limit(SAMPLE_RATE * 60.0);
pub const PLATES_LEN_MAX:  usize = compute_limit(3.0 * 60.0);
pub const BOUNDS_LEN_MAX:  usize = compute_limit(1.0);
pub const STATES_LEN_MAX:  usize = compute_limit(1.0);
pub const ORDERS_LEN_MAX:  usize = compute_limit(1.0);
pub const LOGS_LEN_MAX:    usize = compute_limit(60.0);

unsafe impl bytemuck::Pod for WeightRecord {}
unsafe impl bytemuck::Pod for PlateRecord {}
unsafe impl bytemuck::Pod for BoundsRecord {}
unsafe impl bytemuck::Pod for StateRecord {}
unsafe impl bytemuck::Pod for OrderRecord {}
unsafe impl bytemuck::Pod for LogRecord {}

#[derive(Debug)]
pub struct TelemetryData {
    pub weights: Vec<WeightRecord, WEIGHTS_LEN_MAX>,
    pub plates:  Vec<PlateRecord,  PLATES_LEN_MAX>,
    pub bounds:  Vec<BoundsRecord, BOUNDS_LEN_MAX>,
    pub states:  Vec<StateRecord,  STATES_LEN_MAX>,
    pub orders:  Vec<OrderRecord,  ORDERS_LEN_MAX>,
    pub logs:    Vec<LogRecord,    LOGS_LEN_MAX>,
}

#[repr(C)]
struct TelemetryHeader {
    magic:   u32,
    version: u32,
}

impl TelemetryData {
    pub fn encode<W: Write>(&self, writer: &mut W) -> Result<()> {
        // weights
        writer.write_all(&(self.weights.len() as u32).to_le_bytes())?;
        writer.write_all(cast_slice(&self.weights))?;

        // plates
        writer.write_all(&(self.plates.len() as u32).to_le_bytes())?;
        writer.write_all(cast_slice(&self.plates))?;

        // bounds
        writer.write_all(&(self.bounds.len() as u32).to_le_bytes())?;
        writer.write_all(cast_slice(&self.bounds))?;

        // states
        writer.write_all(&(self.states.len() as u32).to_le_bytes())?;
        writer.write_all(cast_slice(&self.states))?;

        // orders
        writer.write_all(&(self.orders.len() as u32).to_le_bytes())?;
        writer.write_all(cast_slice(&self.orders))?;

        // logs
        writer.write_all(&(self.logs.len() as u32).to_le_bytes())?;
        writer.write_all(cast_slice(&self.logs))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    pub fn my_test() {
        let mut data = TelemetryData {
            weights: heapless::Vec::new(),
            plates: heapless::Vec::new(),
            bounds: heapless::Vec::new(),
            states: heapless::Vec::new(),
            orders: heapless::Vec::new(),
            logs: heapless::Vec::new(),
        };

        // -------------------------
        // Fill with sample data
        // -------------------------
        data.weights.push(WeightRecord {
            weight_0: 10,
            weight_1: 20,
            weight_total: 30,
        }).unwrap();

        data.plates.push(PlateRecord {
            peak: 1,
            drop: 2,
            exit: 3,
        }).unwrap();

        data.bounds.push(BoundsRecord {
            min: 1,
            max: 2,
            desired: 3,
            trigger: 4,
        }).unwrap();

        data.states.push(StateRecord {
            order_id: 0,
            state_id: 0,
        });

        data.orders.push(OrderRecord {
            order_id: 0,
            personnel_id_buf: [0u8; 20],
            personnel_id_len: 0,
            quantity_scrap: 0,
            quantity_good: 0,
            time_start: 0,
            time_end: 0,
            duration: 0,
        });

        data.logs.push(LogRecord {
            level: LogLevel::Debug,
            message_buf: [0u8; 96],
            message_len: 0,
        });

        // -------------------------
        // Encode into memory buffer
        // -------------------------
        let mut buffer = Cursor::new(Vec::new());
        data.encode(&mut buffer).unwrap();

        let encoded_bytes = buffer.into_inner();

        // -------------------------
        // Decode back
        // -------------------------
        let decoded = TelemetryData::decode(&encoded_bytes).unwrap();

        // -------------------------
        // Assertions
        // -------------------------
        assert_eq!(decoded.weights.len(), data.weights.len());
        assert_eq!(decoded.plates.len(), data.plates.len());
        assert_eq!(decoded.bounds.len(), data.bounds.len());
        assert_eq!(decoded.states.len(), data.states.len());
        assert_eq!(decoded.orders.len(), data.orders.len());
        assert_eq!(decoded.logs.len(), data.logs.len());

        assert_eq!(decoded.weights[0].weight_total, 30);
        assert_eq!(decoded.plates[0].peak, 1);
    }
}