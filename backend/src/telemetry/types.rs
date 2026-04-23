use std::{ fmt::Write };

use arc_swap::ArcSwap;

use crate::telemetry::{archive_manager::ArchiveRegistry, segment::DataFragment};

#[allow(unused)]
#[derive(Debug, Default)]
pub struct Shared {
    pub segment_snapshot: ArcSwap<Option<DataFragment>>,
    pub segment_registry: ArcSwap<ArchiveRegistry>
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct DateTime {
    buf: heapless::String<23>,
}

impl DateTime {
    pub fn now() -> Self {
        let now = chrono::Local::now();
        let mut buf = heapless::String::<23>::new();

        let ts = now.format("%Y-%m-%dT%H:%M:%S%.3f");
        write!(&mut buf, "{}", ts).unwrap();

        Self { buf }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.buf.as_bytes()
    }

    pub fn from_bytes(bytes: [u8; 23]) -> Self {
        _ = bytes;
        todo!();
        // let buf: heapless::String<23> = String::from(bytes);
        // Self { buf: () }
    }
}

#[allow(unused)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info  => write!(f, "INFO"),
            LogLevel::Warn  => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Debug => write!(f, "DEBUG"),
        }
    }
}