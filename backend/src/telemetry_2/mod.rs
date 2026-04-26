use std::fs::File;
use std::io;
use std::sync::Arc;

use arrow::array::{Int16Array, RecordBatch, UInt64Array};
use arrow::datatypes::{Field, Schema};
use arrow::datatypes::DataType;
use parquet::arrow::ArrowWriter;
use soa_derive::StructOfArray;

pub type Timeseries = TimeseriesEntryVec;

#[derive(Debug, StructOfArray)]
pub struct TimeseriesEntry {
    timestamp: u64,
    weight_0:  Option<i16>,
    weight_1:  Option<i16>,
}

impl Timeseries {
    pub fn schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::UInt64, false),
            Field::new("weight_0",  DataType::Int16,  true),
            Field::new("weight_1",  DataType::Int16,  true),
        ]))
    }

    pub fn write(&mut self, path: &str) -> io::Result<()> {
        let file = File::create(path)?;
        let schema = Self::schema();

        let timestamp_vec = std::mem::take(&mut self.timestamp);
        let weight_0_vec  = std::mem::take(&mut self.weight_0);
        let weight_1_vec  = std::mem::take(&mut self.weight_1);

        let timestamp = UInt64Array::from(timestamp_vec);
        let weight_0  = Int16Array::from(weight_0_vec);
        let weight_1  = Int16Array::from(weight_1_vec);

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(timestamp),
                Arc::new(weight_0),
                Arc::new(weight_1),
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

#[derive(Debug, StructOfArray)]
pub struct PlateEntry {
    timestamp: u64,
    peak_real: i16,
    peak_post: i16,
}

#[derive(Debug, StructOfArray)]
pub struct WorkorderEntry {
    timestamp:      u64,
    status:         u8,
    workorder_id:   u32,
    worker_id:      u32,
    item_code:      [u8; 50],
    quantity_total: u16,
    quantity_scrap: u16,
    bounds_minimum: i16,
    bounds_maximum: i16,
    bounds_desired: i16,
    bounds_trigger: i16,
}

#[derive(Debug, StructOfArray)]
pub struct LogEntry {
    timestamp: u64,
    category:  LogCategory,
    messages:  String,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LogCategory {
    Debug,
    Info,
    Warning,
    Error,
}