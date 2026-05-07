use duckdb::{types::Value};
use super::QueryArgs;

pub fn create_sql(args: QueryArgs) -> anyhow::Result<(String, Vec<Value>)> {
    let mut conditions = Vec::new();
    let mut params = Vec::new();

    if let Some(from) = args.get_datetime("from")? {
        conditions.push("timestamp >= ?");
        params.push(Value::Text(from.to_string()));
    }

    if let Some(to) = args.get_datetime("to")? {
        conditions.push("timestamp <= ?");
        params.push(Value::Text(to.to_string()));
    }

    let mut sql = String::new();
    sql.push_str("SELECT * FROM logs");
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY timestamp");

    Ok((sql, params))
}