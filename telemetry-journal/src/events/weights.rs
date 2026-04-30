use std::io;

#[derive(Debug)]
pub struct WeightEvent {
    pub weight_0: Option<i16>,
    pub weight_1: Option<i16>,
}

impl WeightEvent {
    const WEIGHT_0_PRESENT: u8 = 1 << 0;
    const WEIGHT_1_PRESENT: u8 = 1 << 1;

    pub fn encode<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        let mut flags = 0u8;

        if self.weight_0.is_some() {
            flags |= Self::WEIGHT_0_PRESENT;
        }
        if self.weight_1.is_some() {
            flags |= Self::WEIGHT_1_PRESENT;
        }

        out[0] = flags;

        let mut idx = 1;

        if let Some(w) = self.weight_0 {
            out[idx..idx + 2].copy_from_slice(&w.to_le_bytes());
            idx += 2;
        }

        if let Some(w) = self.weight_1 {
            out[idx..idx + 2].copy_from_slice(&w.to_le_bytes());
            idx += 2;
        }

        &out[..idx]
    }

    pub fn decode(data: &[u8]) -> io::Result<Self> {
        if data.len() < 1 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "buffer too small",
            ));
        }

        let flags = data[0];
        let mut idx = 1;

        let weight_0 = if flags & Self::WEIGHT_0_PRESENT != 0 {
            if data.len() < idx + 2 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing weight_0",
                ));
            }

            let v = i16::from_le_bytes(data[idx..idx + 2].try_into().unwrap());
            idx += 2;
            Some(v)
        } else {
            None
        };

        let weight_1 = if flags & Self::WEIGHT_1_PRESENT != 0 {
            if data.len() < idx + 2 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing weight_1",
                ));
            }

            let v = i16::from_le_bytes(data[idx..idx + 2].try_into().unwrap());
            // idx += 2;
            Some(v)
        } else {
            None
        };

        Ok(Self {
            weight_0,
            weight_1,
        })
    }
}