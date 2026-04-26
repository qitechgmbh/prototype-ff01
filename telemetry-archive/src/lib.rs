mod schema;
pub use schema::FragmentSchema;
pub use schema::ColumnSchema;
pub use schema::TableSchema;
pub use schema::ColumnType;

mod types;
pub use types::Table;
pub use types::TableDyn;
pub use types::Column;
pub use types::StringColumn;

mod archive;
pub use archive::ArchiveTier;
pub use archive::ArchiveHeader;
pub use archive::Archive;

mod fragment;
pub use fragment::FragmentHeader;
pub use fragment::Fragment;

mod io;

mod manager;

pub const MAGIC:   u32 = 0xB00B135;
pub const VERSION: u32 = 0001_1001;