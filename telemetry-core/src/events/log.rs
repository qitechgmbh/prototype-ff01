use crate::EntryDecodeError;

#[derive(Debug)]
pub struct LogEvent {
    pub category: LogCategory,
    pub message:  String,
}

impl LogEvent {
    pub fn encode<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        let msg_bytes = self.message.as_bytes();
        let len = msg_bytes.len();

        out[0] = self.category.to_u8();

        out[1..5].copy_from_slice(&(len as u32).to_le_bytes());

        out[5..5 + len].copy_from_slice(msg_bytes);

        &out[..5 + len]
    }

    pub fn decode(data: &[u8]) -> Result<Self, EntryDecodeError> {
        if data.len() < 5 {
            return Err(EntryDecodeError::DataIncomplete);
        }

        let category = LogCategory::from_u8(data[0])
            .ok_or_else(|| EntryDecodeError::UnknownLogCategory)?;

        let len = u32::from_le_bytes(data[1..5].try_into().unwrap()) as usize;

        if data.len() < 5 + len {
            return Err(EntryDecodeError::DataIncomplete);
        }

        let msg = std::str::from_utf8(&data[5..5 + len])
            .map_err(|_| EntryDecodeError::InvalidUtf8)?
            .to_string();

        Ok(Self {
            category,
            message: msg,
        })
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum LogCategory {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogCategory {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(LogCategory::Debug),
            1 => Some(LogCategory::Info),
            2 => Some(LogCategory::Warn),
            3 => Some(LogCategory::Error),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}