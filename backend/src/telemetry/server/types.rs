use serde::{Deserialize, Serialize};
use crate::telemetry::segment::Cursor;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    /// List all available batches
    List,

    /// Retrieve data from cache for next batch
    GetLive { batch_id: u64, cursor: Option<Cursor> },

    /// Retrieve batches in a range
    GetRange { from: u64, to: u64 },
}

#[allow(unused)]
#[derive(Serialize, Debug)]
pub enum Response<'a> {
    List { batches: &'a [u64] },
    Live {
        batch_id: u64,
        cursor: Cursor,
        data: &'a [u8],
    },
    NoData,
    OutOfSync { current_batch_id: u64 },
    Range { batches: &'a [u8] },
    InvalidRange,
    NoSuchRequest,
}