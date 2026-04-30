use std::io;

#[derive(Debug)]
pub struct OrderEvent {
    pub started:     bool,
    pub order_id:    u32,
    pub worker_id:   Option<u32>,
    pub scrap_count: Option<u16>,
}

impl OrderEvent {
    const WORKER_PRESENT: u8 = 1 << 0;
    const SCRAP_PRESENT:  u8 = 1 << 1;

    pub fn encode<'a>(&self, out: &'a mut [u8]) -> &'a [u8] {
        let mut flags = 0u8;

        if self.worker_id.is_some() {
            flags |= Self::WORKER_PRESENT;
        }
        if self.scrap_count.is_some() {
            flags |= Self::SCRAP_PRESENT;
        }        

        out[0] = flags;
        out[1] = self.started as u8;

        out[2..6].copy_from_slice(&self.order_id.to_le_bytes());

        let mut idx = 6;

        if let Some(w) = self.worker_id {
            out[idx..idx+4].copy_from_slice(&w.to_le_bytes());
            idx += 4;
        }

        if let Some(s) = self.scrap_count {
            out[idx..idx+2].copy_from_slice(&s.to_le_bytes());
            idx += 2;
        }

        &out[..idx]
    }

    pub fn decode(data: &[u8]) -> io::Result<Self> {
        if data.len() < 6 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "buffer too small",
            ));
        }

        let flags   = data[0];
        let started = data[1] != 0;

        let order_id = u32::from_le_bytes(
            data[2..6].try_into().unwrap()
        );

        let mut idx = 6;

        let worker_id = if flags & Self::WORKER_PRESENT != 0 {
            if data.len() < idx + 4 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing worker_id",
                ));
            }

            let v = u32::from_le_bytes(data[idx..idx+4].try_into().unwrap());
            idx += 4;
            Some(v)
        } else {
            None
        };

        let scrap_count = if flags & Self::SCRAP_PRESENT != 0 {
            if data.len() < idx + 2 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing scrap_count",
                ));
            }

            let v = u16::from_le_bytes(data[idx..idx+2].try_into().unwrap());

            Some(v)
        } else {
            None
        };

        Ok(Self {
            started,
            order_id,
            worker_id,
            scrap_count,
        })
    }
}