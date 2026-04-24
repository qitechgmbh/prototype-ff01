// mod fragment;
mod schema;
pub use schema::Schema;

mod fragment;

#[macro_export]
macro_rules! import_schema {
    ($path:expr) => {{
        const INPUT: &str = include_str!($path);
        $crate::format::Schema::deserialize(INPUT)
    }};
}