mod schema;
pub use schema::FragmentSchema;
pub use schema::TableSchema;
pub use schema::ColumnSchema;

mod tier_registry;
mod fragment_registry;
mod manager;

#[derive(Debug, Clone, Copy, Default)]
pub struct ArchiveTier {
    pub capacity_desired: usize,
    pub capacity_max: usize,
}

#[macro_export]
macro_rules! import_schema {
    ($path:expr) => {{
        const INPUT: &str = include_str!($path);
        $crate::FragmentSchema::deserialize(INPUT)
    }};
}