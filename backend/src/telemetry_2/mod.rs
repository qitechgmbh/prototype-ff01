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

#[derive(Debug, StructOfArray)]
pub struct PlateEntry {
    timestamp: u64,
    peak_real: i16,
    peak_post: i16,
}

#[derive(Debug, StructOfArray)]
pub struct WorkorderRecord {
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