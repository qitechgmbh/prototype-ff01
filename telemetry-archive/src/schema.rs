#[derive(Debug, Clone, Copy)]
pub struct FragmentSchema {
    pub tables: &'static [TableSchema],
}

#[derive(Debug, Clone, Copy)]
pub struct TableSchema {
    pub name: &'static str,
    pub columns: &'static [ColumnSchema],
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnSchema {
    pub name: &'static str,
    pub r#type: ColumnType,
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnType {
    Unsigned8,
    Unsigned16,
    Unsigned32,
    Unsigned64,
    Signed8,
    Signed16,
    Signed32,
    Signed64,
    Float32,
    Float64,
    String,
}