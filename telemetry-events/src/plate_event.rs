#[derive(Debug, Clone)]
pub struct PlateEvent {
    pub order_id: Option<u32>,
    pub peak: i16,
    pub real: i16,
}

impl PlateEvent {
    pub fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let mut flags: u8 = 0;
        if self.order_id.is_none() { flags |= 1 << 0; }

        buf[0] = flags;
        let mut i = 1;

        // order_id
        if let Some(v) = self.order_id {
            buf[i..i + 4].copy_from_slice(&v.to_le_bytes());
            i += 4;
        }

        buf[i..i + 2].copy_from_slice(&self.peak.to_le_bytes());
        i += 2;

        buf[i..i + 2].copy_from_slice(&self.real.to_le_bytes());
        i += 2;

        &buf[..i]
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        let flags = buf[0];
        let mut i = 1;

        // order_id
        let order_id = if flags & (1 << 0) == 0 {
            if buf.len() < i + 4 { return None; }
            let v = u32::from_le_bytes(buf[i..i + 4].try_into().unwrap());
            i += 4;
            Some(v)
        } else {
            None
        };

        let peak = i16::from_le_bytes(buf[i..i + 2].try_into().unwrap());
        i += 2;

        let real = i16::from_le_bytes(buf[i..i + 2].try_into().unwrap());
        //i += 2;

        Some(Self { order_id, peak, real })
    }
}