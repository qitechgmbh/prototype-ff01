pub const NAME_LEN_MAX: usize = 128;
pub const TABLES_LEN_MAX: usize = 16;
pub const COLUMNS_LEN_MAX: usize = 32;

#[derive(Debug, Clone)]
pub struct Schema {
    pub tables: heapless::Vec<Table, TABLES_LEN_MAX>,
}

impl Schema {
    pub fn deserialize(input: &str) -> Schema {
        let mut schema = Schema {
            tables: heapless::Vec::new(),
        };

        let mut current: Option<Table> = None;

        for line in input.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // section start: [Weights]
            if line.starts_with('[') && line.ends_with(']') {
                // flush previous table
                if let Some(t) = current.take() {
                    schema.tables.push(t).unwrap();
                }

                let name = &line[1..line.len() - 1];

                let mut table = Table {
                    name: heapless::String::new(),
                    columns: heapless::Vec::new(),
                };

                table.name.push_str(name).unwrap();
                current = Some(table);
                continue;
            }

            // key=value
            if let Some((key, ty)) = line.split_once('=') {
                let table = current.as_mut().unwrap();

                let mut name = heapless::String::new();
                name.push_str(key.trim()).unwrap();

                table.columns.push(Column {
                    name,
                    r#type: ColumnType::deserialize(ty.trim()),
                }).unwrap();
            }
        }

        // flush last table
        if let Some(t) = current {
            schema.tables.push(t).unwrap();
        }

        schema
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: heapless::String<NAME_LEN_MAX>,
    pub columns: heapless::Vec<Column, COLUMNS_LEN_MAX>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: heapless::String<NAME_LEN_MAX>,
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
    String { max_len: u16 },
}

impl ColumnType {
    fn deserialize(s: &str) -> ColumnType {
        match s {
            "u8"  => ColumnType::Unsigned8,
            "u16" => ColumnType::Unsigned16,
            "u32" => ColumnType::Unsigned32,
            "u64" => ColumnType::Unsigned64,
            "i8"  => ColumnType::Signed8,
            "i16" => ColumnType::Signed16,
            "i32" => ColumnType::Signed32,
            "i64" => ColumnType::Signed64,
            "f32" => ColumnType::Float32,
            "f64" => ColumnType::Float64,
            _ => ColumnType::String { max_len: 128 },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::import_schema;

    #[test]
    fn parse_static_schema() {
        let schema = import_schema!("../../../schemas/telemetry.ini");

        println!("schema: {:?}", schema);

        assert_eq!(schema.tables.len(), 3);

        let weights = &schema.tables[0];
        assert_eq!(weights.name.as_str(), "Weights");
        assert_eq!(weights.columns.len(), 2);

        let plates = &schema.tables[1];
        assert_eq!(plates.name.as_str(), "Plates");
        assert_eq!(plates.columns.len(), 2);

        let bounds = &schema.tables[2];
        assert_eq!(bounds.name.as_str(), "Bounds");
        assert_eq!(bounds.columns.len(), 4);
    }
}