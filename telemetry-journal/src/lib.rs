use std::io::Write;
use std::io::{self, Read};

mod events;
pub use events::Event;
pub use events::WeightEvent;
pub use events::PlateEvent;
pub use events::OrderEvent;
pub use events::LogEvent;
pub use events::LogCategory;

mod archive;
pub use archive::scan_and_process;

pub mod journal;

#[derive(Debug)]
pub struct WalEntry {
    pub timestamp: u64,
    pub event:     Event,
}

impl WalEntry {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut buf = [0u8; 256];
        let event = self.event.encode(&mut buf);
        let len = &[(event.len() + size_of::<u64>()) as u8];
        let ts = &self.timestamp.to_le_bytes();

        writer.write_all(len)?;
        writer.write_all(ts)?;
        writer.write_all(event)?;

        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Option<Self>> {
        use io::ErrorKind;

        let mut buf_len  = [0u8; 1];
        match reader.read_exact(&mut buf_len) {
            Ok(v) => v,
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e),
        };
        let len = u8::from_le_bytes(buf_len);

        let mut buf_ts = [0u8; 8];
        reader.read_exact(&mut buf_ts)?;
        let timestamp = u64::from_le_bytes(buf_ts);

        let mut event_buf = [0u8; 256];
        let event_len = len as usize - size_of::<u64>();
        reader.read_exact(&mut event_buf[0..event_len])?;
        let event = Event::decode(&event_buf)?;

        Ok(Some(WalEntry { timestamp, event }))
    }
}