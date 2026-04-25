#[derive(Debug, Clone)]
pub struct Fragment {
    pub range:  u64,
    pub count:  u64,
    pub tables: Vec<Table>
}

#[derive(Debug, Clone)]
pub struct Table {
    columns: Vec<HeapColumn>,
}

#[derive(Debug, Clone)]
pub enum HeapColumn {
    Timestamp(Vec<u64>),

    Unsigned8(Vec<u8>),
    Unsigned16(Vec<u16>),
    Unsigned32(Vec<u32>),
    Unsigned64(Vec<u64>),

    Signed8(Vec<i8>),
    Signed16(Vec<i16>),
    Signed32(Vec<i32>),
    Signed64(Vec<i64>),

    Float32(Vec<f32>),
    Float64(Vec<f64>),

    String(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum HeaplessColumn<const N: usize> {
    Timestamp(heapless::Vec<u64, N>),

    Unsigned8(Vec<u8>),
    Unsigned16(Vec<u16>),
    Unsigned32(Vec<u32>),
    Unsigned64(Vec<u64>),

    Signed8(Vec<i8>),
    Signed16(Vec<i16>),
    Signed32(Vec<i32>),
    Signed64(Vec<i64>),

    Float32(Vec<f32>),
    Float64(Vec<f64>),

    String(Vec<String>),
}