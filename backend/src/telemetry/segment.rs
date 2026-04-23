use std::{io::{self, Read, Write}, time::{SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};

use crate::telemetry::{binary::Fragment, types::DateTime};

// 12 samples per second for 5 minutes + 20% headroom
pub const MAX_SAMPLES: usize = 12 * 60 * 6;

type NumericSamples<T> = heapless::Vec<T, MAX_SAMPLES>;
type StringSamples<const MAX_SIZE: usize> = heapless::Vec<heapless::String<MAX_SIZE>, MAX_SAMPLES>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    weights: u32,
}

#[allow(unused)]
#[derive(Debug)]
pub struct DataFragment {
    range:   (u64, u64),
    weights: WeightRecords,
    plates:  PlateRecords,
    bounds:  BoundsRecords,
    states:  StateRecords,
    // orders:  OrderRecords,
}

impl Fragment for DataFragment {
    fn range(&self) -> (u64, u64) {
        self.range
    }

    fn encode_body<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.weights.write(writer)?;
        self.plates.write(writer)?;
        self.bounds.write(writer)?;
        self.states.write(writer)?;
        // self.orders.write(writer)?;
        Ok(())
    }
    
    fn decode<R: Read>(reader: &mut R, range: (u64, u64)) -> io::Result<Self> {
        let weights = WeightRecords::read(reader)?;
        let plates  = PlateRecords::read(reader)?;
        let bounds  = BoundsRecords::read(reader)?;
        let states  = StateRecords::read(reader)?;
        // let orders  = OrderRecords::read(reader)?;

        Ok(Self { 
            range, 
            weights, 
            plates, 
            bounds, 
            states, 
            // orders 
        })
    }
}

impl DataFragment {
    pub fn new(now: SystemTime) -> Self {
        let millis = now
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self { 
            range:   (millis, millis),
            weights: Default::default(),
            plates:  Default::default(),
            bounds:  Default::default(),
            states:  Default::default(),
            // orders:  Default::default(),
        }
    }

    pub fn append_weight(&mut self, dt: u32, w0: i16, w1: i16) -> Result<(), ()> {
        self.weights.dt.push(dt).map_err(|_| ())?;
        self.weights.w0.push(w0).map_err(|_| ())?;
        self.weights.w1.push(w1).map_err(|_| ())?;
        Ok(())
    }

        pub fn append_plate(&mut self, dt: u32, peak: i16, avg: i16) {
        self.plates.dt.push(dt).expect("Should never be full");
        self.plates.peak.push(peak).expect("Should never be full");
        self.plates.avg.push(avg).expect("Should never be full");
    }

    pub fn append_bounds(
        &mut self,
        dt: u32,
        min: i16,
        max: i16,
        desired: i16,
        trigger: i16,
    ) {
        self.bounds.dt.push(dt).expect("Should never be full");
        self.bounds.min.push(min).expect("Should never be full");
        self.bounds.max.push(max).expect("Should never be full");
        self.bounds.desired.push(desired).expect("Should never be full");
        self.bounds.trigger.push(trigger).expect("Should never be full");
    }

    pub fn append_state(&mut self, order_id: u32, state_id: u32) {
        self.states.order_id.push(order_id).expect("Should never be full");
        self.states.state_id.push(state_id).expect("Should never be full");
    }

    /*
    pub fn append_order(
        &mut self,
        order_id: u32,
        personnel_id: &str,
        quantity_scrap: u16,
        quantity_good: u16,
        time_start: &str,
        time_end: &str,
        duration: u32,
    ) {
        self.orders.order_id.push(order_id).expect("Should never be full");

        let mut personnel = heapless::String::<20>::new();
        personnel.push_str(personnel_id).expect("personnel_id too long");
        self.orders.personnel_id.push(personnel).expect("Should never be full");

        self.orders.quantity_scrap.push(quantity_scrap).expect("Should never be full");
        self.orders.quantity_good.push(quantity_good).expect("Should never be full");

        let mut start = heapless::String::<10>::new();
        start.push_str(time_start).expect("time_start too long");
        self.orders.time_start.push(start).expect("Should never be full");

        let mut end = heapless::String::<10>::new();
        end.push_str(time_end).expect("time_end too long");
        self.orders.time_end.push(end).expect("Should never be full");

        self.orders.duration.push(duration).expect("Should never be full");
    }
    */
}

#[allow(unused)]
#[derive(Debug)]
pub struct ActiveWorkorder {
    order_id:    u32,
    worker_id:   u32,
    start_time:  DateTime,
    plate_count: u16,
}

impl ActiveWorkorder {
    #[allow(unused)]
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.order_id.to_le_bytes())?;
        writer.write_all(&self.worker_id.to_le_bytes())?;
        writer.write_all(&self.start_time.as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct WeightRecords {
    pub dt: NumericSamples<u32>,
    pub w0: NumericSamples<i16>,
    pub w1: NumericSamples<i16>,
}

impl WeightRecords {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // number of samples
        let len = self.dt.len();
        assert_eq!(len, self.w0.len());
        assert_eq!(len, self.w1.len());

        writer.write_all(&len.to_le_bytes())?;

        // columnar data (raw memory copy)
        let dt = bytemuck::cast_slice(&self.dt);
        let w0 = bytemuck::cast_slice(&self.w0);
        let w1 = bytemuck::cast_slice(&self.w1);

        writer.write_all(dt)?;
        writer.write_all(w0)?;
        writer.write_all(w1)?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut len_buf = [0u8; 8];
        reader.read_exact(&mut len_buf)?;
        let len = usize::from_le_bytes(len_buf);

        let mut dt = NumericSamples::default();
        let mut w0 = NumericSamples::default();
        let mut w1 = NumericSamples::default();

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            dt.push(u32::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            w0.push(i16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            w1.push(i16::from_le_bytes(b)).unwrap();
        }

        Ok(Self { dt, w0, w1 })
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlateRecords {
    pub dt:   NumericSamples<u32>,
    pub peak: NumericSamples<i16>,
    pub avg:  NumericSamples<i16>,
}

impl PlateRecords {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // number of samples
        let len = self.dt.len() as u16;
        writer.write_all(&len.to_le_bytes())?;

        // columnar data (raw memory copy)
        let dt   = bytemuck::cast_slice(&self.dt);
        let peak = bytemuck::cast_slice(&self.peak);
        let avg  = bytemuck::cast_slice(&self.avg);

        writer.write_all(dt)?;
        writer.write_all(peak)?;
        writer.write_all(avg)?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let len = u16::from_le_bytes(len_buf) as usize;

        let mut dt = NumericSamples::default();
        let mut peak = NumericSamples::default();
        let mut avg = NumericSamples::default();

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            dt.push(u32::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            peak.push(i16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            avg.push(i16::from_le_bytes(b)).unwrap();
        }

        Ok(Self { dt, peak, avg })
    }
}

#[derive(Debug, Clone, Default)]
pub struct BoundsRecords {
    pub dt:      NumericSamples<u32>,
    pub min:     NumericSamples<i16>,
    pub max:     NumericSamples<i16>,
    pub desired: NumericSamples<i16>,
    pub trigger: NumericSamples<i16>,
}

impl BoundsRecords {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // number of samples
        let len = self.dt.len() as u16;
        writer.write_all(&len.to_le_bytes())?;

        // columnar data (raw memory copy)
        let dt      = bytemuck::cast_slice(&self.dt);
        let min     = bytemuck::cast_slice(&self.min);
        let max     = bytemuck::cast_slice(&self.max);
        let desired = bytemuck::cast_slice(&self.desired);
        let trigger = bytemuck::cast_slice(&self.trigger);

        writer.write_all(dt)?;
        writer.write_all(min)?;
        writer.write_all(max)?;
        writer.write_all(desired)?;
        writer.write_all(trigger)?;

        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let len = u16::from_le_bytes(len_buf) as usize;

        let mut dt = NumericSamples::default();
        let mut min = NumericSamples::default();
        let mut max = NumericSamples::default();
        let mut desired = NumericSamples::default();
        let mut trigger = NumericSamples::default();

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            dt.push(u32::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            min.push(i16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            max.push(i16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            desired.push(i16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            trigger.push(i16::from_le_bytes(b)).unwrap();
        }

        Ok(Self { dt, min, max, desired, trigger })
    }
}

#[derive(Debug, Clone, Default)]
pub struct StateRecords {
    pub order_id: NumericSamples<u32>,
    pub state_id: NumericSamples<u32>,
}

impl StateRecords {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.order_id.len() as u16;
        writer.write_all(&len.to_le_bytes())?;

        let order_id = bytemuck::cast_slice(&self.order_id);
        let state_id = bytemuck::cast_slice(&self.state_id);

        writer.write_all(order_id)?;
        writer.write_all(state_id)?;

        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let len = u16::from_le_bytes(len_buf) as usize;

        let mut order_id = NumericSamples::default();
        let mut state_id = NumericSamples::default();

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            order_id.push(u32::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            state_id.push(u32::from_le_bytes(b)).unwrap();
        }

        Ok(Self { order_id, state_id })
    }
}

/* 
#[derive(Debug, Clone, Default)]
pub struct OrderRecords {
    pub order_id:       NumericSamples<u32>,
    pub personnel_id:   StringSamples<20>,
    pub quantity_scrap: NumericSamples<u16>,
    pub quantity_good:  NumericSamples<u16>,
    pub time_start:     StringSamples<10>,
    pub time_end:       StringSamples<10>,
    pub duration:       NumericSamples<u32>,
}

impl OrderRecords {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let len = self.order_id.len() as u16;
        writer.write_all(&len.to_le_bytes())?;

        let order_id = bytemuck::cast_slice(&self.order_id);
        let quantity_scrap = bytemuck::cast_slice(&self.quantity_scrap);
        let quantity_good = bytemuck::cast_slice(&self.quantity_good);
        let duration = bytemuck::cast_slice(&self.duration);

        writer.write_all(order_id)?;
        writer.write_all(quantity_scrap)?;
        writer.write_all(quantity_good)?;
        writer.write_all(duration)?;

        for s in &self.personnel_id {
            let bytes = s.as_bytes();
            writer.write_all(bytes)?;
            if bytes.len() < 20 {
                writer.write_all(&vec![0u8; 20 - bytes.len()])?;
            }
        }

        for s in &self.time_start {
            let bytes = s.as_bytes();
            writer.write_all(bytes)?;
            if bytes.len() < 10 {
                writer.write_all(&vec![0u8; 10 - bytes.len()])?;
            }
        }

        for s in &self.time_end {
            let bytes = s.as_bytes();
            writer.write_all(bytes)?;
            if bytes.len() < 10 {
                writer.write_all(&vec![0u8; 10 - bytes.len()])?;
            }
        }

        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut len_buf = [0u8; 2];
        reader.read_exact(&mut len_buf)?;
        let len = u16::from_le_bytes(len_buf) as usize;

        let mut order_id = NumericSamples::default();
        let mut quantity_scrap = NumericSamples::default();
        let mut quantity_good = NumericSamples::default();
        let mut duration = NumericSamples::default();

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            order_id.push(u32::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            quantity_scrap.push(u16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 2];
            reader.read_exact(&mut b)?;
            quantity_good.push(u16::from_le_bytes(b)).unwrap();
        }

        for _ in 0..len {
            let mut b = [0u8; 4];
            reader.read_exact(&mut b)?;
            duration.push(u32::from_le_bytes(b)).unwrap();
        }

        let mut personnel_id = StringSamples::default();
        let mut time_start = StringSamples::default();
        let mut time_end = StringSamples::default();

        for _ in 0..len {
            let mut buf = [0u8; 20];
            reader.read_exact(&mut buf)?;
            let s = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_string();

            personnel_id.push(heapless::String::try_from(s).unwrap()).unwrap();
        }

        for _ in 0..len {
            let mut buf = [0u8; 10];
            reader.read_exact(&mut buf)?;
            let s = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_string();

            time_start.push(heapless::String::try_from(s).unwrap()).unwrap();
        }

        for _ in 0..len {
            let mut buf = [0u8; 10];
            reader.read_exact(&mut buf)?;
            let s = String::from_utf8_lossy(&buf)
                .trim_end_matches('\0')
                .to_string();
            time_end.push(heapless::String::try_from(s).unwrap()).unwrap();
        }

        Ok(Self {
            order_id,
            personnel_id,
            quantity_scrap,
            quantity_good,
            time_start,
            time_end,
            duration,
        })
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LogRecords { // will be encoded manually not via copy
    pub level: LogLevel,
    pub message_buf: [u8; 256],
    pub message_len: u8,
}
    */