use duckdb::{types::Value};
use super::QueryArgs;

pub fn create_sql(args: QueryArgs) -> anyhow::Result<(String, Vec<Value>)> {
    let mut conditions = Vec::new();
    let mut bindings   = Vec::new();

    if let Some(order_id) = args.get_int::<u32>("order_id")? {
        conditions.push("order_id = ?");
        bindings.push(Value::UInt(order_id));
    }

    if let Some(from) = args.get_datetime("from")? {
        conditions.push("timestamp >= ?");
        bindings.push(Value::Text(from.to_string()));
    }

    if let Some(to) = args.get_datetime("to")? {
        conditions.push("timestamp <= ?");
        bindings.push(Value::Text(to.to_string()));
    }

    let mut sql = String::new();
    sql.push_str("SELECT * FROM weights");
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY timestamp");

    Ok((sql, bindings))
}