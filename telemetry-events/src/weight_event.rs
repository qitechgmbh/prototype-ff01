#[derive(Debug, Clone)]
pub struct WeightEvent {
    pub order_id: Option<u32>,
    pub weight_0: Option<i16>,
    pub weight_1: Option<i16>,
}

impl WeightEvent {
    pub fn encode<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let mut flags: u8 = 0;
        if self.order_id.is_none() { flags |= 1 << 0; }
        if self.weight_0.is_none() { flags |= 1 << 1; }
        if self.weight_1.is_none() { flags |= 1 << 2; }
        buf[0] = flags;

        let mut i = 1;

        // order_id
        if let Some(v) = self.order_id {
            buf[i..i + 4].copy_from_slice(&v.to_le_bytes());
            i += 4;
        }

        // weight 0
        if let Some(v) = self.weight_0 {
            buf[i..i + 2].copy_from_slice(&v.to_le_bytes());
            i += 2;
        }

        // weight 1
        if let Some(v) = self.weight_1 {
            buf[i..i + 2].copy_from_slice(&v.to_le_bytes());
            i += 2;
        }

        &buf[..i]
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.is_empty() {
            return None;
        }

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

        // weight_0
        let weight_0 = if flags & (1 << 1) == 0 {
            if buf.len() < i + 2 { return None; }
            let v = i16::from_le_bytes(buf[i..i + 2].try_into().unwrap());
            i += 2;
            Some(v)
        } else {
            None
        };

        // weight_1
        let weight_1 = if flags & (1 << 2) == 0 {
            if buf.len() < i + 2 { return None; }
            let v = i16::from_le_bytes(buf[i..i + 2].try_into().unwrap());
            i += 2;
            Some(v)
        } else {
            None
        };

        if i != buf.len() {
            return None;
        }

        Some(Self { order_id, weight_0, weight_1 })
    }
}