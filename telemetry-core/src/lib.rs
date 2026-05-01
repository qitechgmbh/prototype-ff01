mod events;
use std::fmt;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Local;
pub use events::Event;
pub use events::WeightEvent;
pub use events::PlateEvent;
pub use events::OrderEvent;
pub use events::LogEvent;
pub use events::LogCategory;

#[derive(Debug)]
pub struct Entry {
    pub timestamp: u64,
    pub event:     Event,
}

impl Entry {
    pub fn encode<'a>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EntryEncoodeError> {
        let event_len = self.event.encode(&mut buf[9..]).len();

        let total_len = 8 + event_len;
        if buf.len() < total_len + 1 {
            return Err(EntryEncoodeError::BufferTooSmall);
        }

        buf[0] = total_len as u8;
        buf[1..9].copy_from_slice(&self.timestamp.to_le_bytes());

        Ok(&buf[..1 + total_len])
    }

    pub fn decode(buf: &[u8]) -> Result<Self, EntryDecodeError> {
        let len = buf[0] as usize;

        if buf.len() < len + 3 { // +crc
            return Err(EntryDecodeError::DataIncomplete);
        }

        let payload = &buf[1..len];

        let timestamp = u64::from_le_bytes(payload[0..8].try_into().unwrap());
        let event     = Event::decode(&payload[8..])?;

        // let crc_expected = u16::from_le_bytes(buf[len..len+2].try_into().unwrap());
        // let crc_actual = compute_crc(&buf[1..len]);
        // if crc_expected != crc_actual {
        //     return Err(EntryDecodeError::Corrupt);
        // }

        Ok(Self { timestamp, event })
    }
}

#[derive(Debug)]
pub enum EntryEncoodeError {
    BufferTooSmall,
}

impl fmt::Display for EntryEncoodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryEncoodeError::BufferTooSmall => write!(f, "BufferTooSmall"),
        }
    }
}

impl std::error::Error for EntryEncoodeError {}

#[derive(Debug)]
pub enum EntryDecodeError {
    DataIncomplete,
    UnknownTag,
    InvalidUtf8,
    UnknownLogCategory,
}

impl fmt::Display for EntryDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryDecodeError::DataIncomplete => write!(f, "incomplete entry data"),
            EntryDecodeError::UnknownTag => write!(f, "unknown tag in entry"),
            EntryDecodeError::InvalidUtf8 => write!(f, "invalid UTF-8 in entry"),
            EntryDecodeError::UnknownLogCategory => write!(f, "unknown log category"),
        }
    }
}

impl std::error::Error for EntryDecodeError {}

pub fn wal_path_from_date(dir_logs: &PathBuf, dt: DateTime<Local>) -> PathBuf {
    dir_logs.join(format!("{}.wal", dt.format("%Y%m%d")))
}