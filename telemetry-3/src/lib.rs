use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use arrow::array::{Int16Array, Int16Builder, UInt64Array, UInt64Builder};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use crossbeam::channel::{Receiver, Sender};
use parquet::arrow::ArrowWriter;

pub trait TelemetryTable {
    type Entry;
}

pub trait TelemetryData {
    type TimeSeries: TelemetryTable;
    type States: TelemetryTable;
}

pub enum TelemetryCollectorAction<T: TelemetryData> {
    AppendTimeSeries(<T::TimeSeries as TelemetryTable>::Entry),
    AppendStates(<T::States as TelemetryTable>::Entry),
    AppendLog(LogEntry),
    Flush,
}

// send same messages to manager and server
// is double data but then its up to date
// in both without any sync mechanism

#[derive(Debug)]
pub struct TelemetryDataDistributor<T: TelemetryData> {
    tx_collectors: Vec<Sender<TelemetryCollectorAction<T>>>
}

impl<T: TelemetryData> TelemetryDataDistributor<T> {
    pub fn record_timeseries(&mut self) {
        
    }

    pub fn record_states(&mut self) {
        
    }

    pub fn record_log(&mut self) {
        
    }
}

// hot data snapshot (updated every second) 2 mb. 
// registry snapshot (updated everytime filesystem changes)

#[derive(Debug)]
pub struct TelemetryDataCollector<T: TelemetryData> {
    rx:   Receiver<TelemetryCollectorAction<T>>,
    logs: LogsTable,
}

#[derive(Debug)]
pub struct TelemetryManager<T: TelemetryData> {
    root: PathBuf,
    collector: TelemetryDataCollector<T>,
}

// 

// ARCHIVE(Timeseries)
// ARCHIVE(States)
// ARCHIVE(Logs)
// LIVEDATA(Timeseries)
// LIVEDATA(States)
// LIVEDATA(Logs)

// if we receive 

pub struct Shared<T: TelemetryData> {
    snapshot_timeseries: ArcSwap<T::TimeSeries>,
    snapshot_states:     ArcSwap<T::States>,
    snapshot_registry:   ArcSwap<T::TimeSeries>,
}

#[derive(Debug)]
pub struct LogEntry {
    timestamp:       u64,
    category:        LogCategory,
    message_offsets: String,
}

#[derive(Debug)]
pub struct LogsTable {
    timestamp:       Vec<u64>,
    category:        Vec<u8>,
    message_buf:     Vec<u8>,
    message_offsets: Vec<u32>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LogCategory {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct TimeSeriesItem {
    pub timestamp:  u64,
    pub weight_0:   Option<i16>,
    pub weight_1:   Option<i16>,
    pub plate_real: Option<i16>,
    pub plate_post: Option<i16>,
}

#[derive(Debug)]
pub struct TimeSeries {
    pub timestamp:  UInt64Builder,
    pub weight_0:   Int16Builder,
    pub weight_1:   Int16Builder,
    pub plate_real: Int16Builder,
    pub plate_post: Int16Builder,
}

impl TimeSeries {
    pub fn schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("timestamp",  DataType::UInt64, false),
            Field::new("weight_0",   DataType::Int16,  true),
            Field::new("weight_1",   DataType::Int16,  true),
            Field::new("plate_real", DataType::Int16,  true),
            Field::new("plate_post", DataType::Int16,  true),
        ]))
    }

    pub fn push(&mut self, item: LogsTable) {
        self.timestamp.append_value(item.timestamp);
        self.weight_0.append_option(item.weight_0);
        self.weight_1.append_option(item.weight_1);
        self.plate_real.append_option(item.plate_real);
        self.plate_post.append_option(item.plate_post);
    }

    pub fn write(&mut self, path: &str) -> io::Result<()> {
        let file = File::create(path)?;

        let schema = Self::schema();

        let timestamp  = UInt64Array::from(self.timestamp.finish());
        let weight_0   = Int16Array::from(self.weight_0.finish());
        let weight_1   = Int16Array::from(self.weight_1.finish());
        let plate_real = Int16Array::from(self.plate_real.finish());
        let plate_post = Int16Array::from(self.plate_post.finish());

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(timestamp),
                Arc::new(weight_0),
                Arc::new(weight_1),
                Arc::new(plate_real),
                Arc::new(plate_post),
            ],
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut writer = ArrowWriter::try_new(file, schema, None)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writer.write(&batch)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        writer.close()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_example_parquet_variant() {
        let mut timestamp  = UInt64Builder::new();
        let mut weight_0   = Int16Builder::new();
        let mut weight_1   = Int16Builder::new();
        let mut plate_real = Int16Builder::new();
        let mut plate_post = Int16Builder::new();

        // simulate structured telemetry pattern
        for i in 0..100 {
            timestamp.append_value(i as u64);
            weight_0.append_value(i);
            weight_1.append_value(i * 2);
            plate_real.append_value(100 + (i % 5));
            plate_post.append_value(200 + (i % 3));
        }

        let mut ts = TimeSeries {
            timestamp,
            weight_0,
            weight_1,
            plate_real,
            plate_post,
        };

        let path = "/home/entity/work/qitech/prototype-ff01/example2.parquet";
        ts.write(path).expect("X");
    }
}