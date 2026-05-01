use crate::EntryDecodeError;

#[derive(Debug)]
pub struct PlateEvent {
    pub peak: i16,
    pub real: i16,
}

impl PlateEvent {
    pub fn encode<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        out[0..2].copy_from_slice(&self.peak.to_le_bytes());
        out[2..4].copy_from_slice(&self.real.to_le_bytes());
        &out[..4]
    }

    pub fn decode(data: &[u8]) -> Result<Self, EntryDecodeError> {
        if data.len() < 4 {
            return Err(EntryDecodeError::DataIncomplete);
        }

        let peak = i16::from_le_bytes(data[0..2].try_into().unwrap());
        let real = i16::from_le_bytes(data[2..4].try_into().unwrap());

        Ok(Self { peak, real })
    }
}