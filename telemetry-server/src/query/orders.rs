use std::{collections::HashMap, net::TcpStream};

use anyhow::anyhow;
use arrow::ipc::writer::StreamWriter;
use duckdb::{Connection, types::Value};

use crate::query::{args::QueryArgs, http};

const ORDER_BY_ALLOWED: [&str; 7] = [
    "order_id",
    "worker_id",
    "created_at",
    "closed_at",
    "quantity_good",
    "quantity_scrap",
    "status",
];

pub fn create_sql(args: QueryArgs) -> anyhow::Result<(String, Vec<Value>)> {
    let mut params = Vec::new();

    let mut sql = String::new();
    sql.push_str("SELECT * FROM orders");

    // order by
    let order_by = args.get_csv("order_by")?;
    if order_by.len() > 0 {
        let mut param = String::new();
        sql.push_str("ORDER BY ?");
        
        for field in order_by {
            if !ORDER_BY_ALLOWED.contains(&field.as_str()) {
                return Err(anyhow!("Unsupported order_by field: {}", field));
            }

            param.push_str(&field);
            param.push_str(",");
        }

        param.pop();
        params.push(Value::Text(param));
    }

    // where 
    let r#where = args.get_csv("where")?;
    if r#where.len() > 0 {
        let mut param = String::new();
        sql.push_str("WHERE ?");
        
        for field in order_by {
            if !ORDER_BY_ALLOWED.contains(&field.as_str()) {
                return Err(anyhow!("Unsupported order_by field: {}", field));
            }

            param.push_str(&field);
            param.push_str(",");
        }

        param.pop();
        params.push(Value::Text(param));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    Ok((sql, params))
}

pub fn handle(
    db_path: &String,
    args: HashMap<&str, &str>,
    stream: &mut TcpStream
) -> anyhow::Result<()> {
    let connection = Connection::open(db_path)?;

    let sql = build_sql(args)?;

    let mut statement = connection.prepare(&sql)?;

    http::start_stream(stream)?;

    for batch in statement.query_arrow([])? {
        let mut buffer = Vec::new();
        {
            let schema = &batch.schema();
            let mut writer = StreamWriter::try_new(&mut buffer, schema)?;
            writer.write(&batch)?;
            writer.finish()?;
        }

        http::write_batch(stream, &buffer)?;
    }

    http::finish_stream(stream)?;
    Ok(())
}

fn build_sql(query: HashMap<&str, &str>) -> anyhow::Result<String> {
    let order_by = match query.get("order_by") {
        Some(v) => {
            let value = parse_order_by(v)?;
            Some(value)
        }
        None => None,
    };

    // allowed keywords: 
    // order_id, worker_id, created_at, closed_at, quantity_good, quantity_scrap, status
    let r#where = match query.get("where") {
        Some(v) => {
            let value = parse_order_by(v)?;
            Some(value)
        }
        None => None,
    };

    let mut sql = format!("SELECT * FROM orders");

    // allowed keywords: 
    // order_id, worker_id, created_at, closed_at, quantity_good, quantity_scrap, status
    let order_by = query.get("order_by");
    // $order_by=order_id, x, x

    // $order_by=created_at 
    // $order_by=closed_at 
    // $where=

    let mut conditions = vec![];

    if let Some(order_id) = order_id {
        conditions.push(format!("order_id = {}", order_id));
    }

    if let Some(from) = from_ts {
        conditions.push(format!("timestamp >= {}", from));
    }

    if let Some(to) = to_ts {
        conditions.push(format!("timestamp <= {}", to));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY timestamp");

    Ok(sql)
}

fn parse_order_by(input: &str) -> anyhow::Result<String> {
    const ORDER_BY_ALLOWED: &[&str] = &[
        "order_id",
        "worker_id",
        "created_at",
        "closed_at",
        "quantity_good",
        "quantity_scrap",
        "status",
    ];

    let mut parts_out = Vec::new();

    for part in input.split(',') {
        let part = part.trim();

        if part.is_empty() {
            return Err(anyhow!("Empty order_by field"));
        }

        // support optional ASC/DESC
        let mut tokens = part.split_whitespace();

        let field = tokens.next().ok_or(anyhow!("Missing field"))?;

        if !ORDER_BY_ALLOWED.contains(&field) {
            return Err(anyhow!("Invalid order_by field: {}", field));
        }

        let direction = match tokens.next() {
            Some("ASC") | None => "ASC",
            Some("DESC") => "DESC",
            Some(_) => return Err(anyhow!("Invalid sort direction")),
        };

        // no extra tokens allowed
        if tokens.next().is_some() {
            return Err(anyhow!("Too many tokens in order_by"));
        }

        parts_out.push(format!("{} {}", field, direction));
    }

    Ok(parts_out.join(", "))
}

const ALLOWED_OPS: &[&str] = &[
    "=",
    "!=",
    ">",
    "<",
    ">=",
    "<=",
];

fn parse_where(input: &str) -> anyhow::Result<String> {
    const FIELDS: &[&str] = &[
        "order_id",
        "worker_id",
        "created_at",
        "closed_at",
        "quantity_good",
        "quantity_scrap",
        "status",
    ];

    const OPS: &[&str] = &["!=", ">=", "<=", "=", ">", "<"];

    let mut out = String::new();

    for (i, cond) in input.split(',').enumerate() {
        let cond = cond.trim();

        if cond.is_empty() {
            anyhow::bail!("Empty condition");
        }

        let mut op_found = None;

        for op in OPS {
            if let Some(idx) = cond.find(op) {
                op_found = Some((idx, *op));
                break;
            }
        }

        let (idx, op) = op_found.ok_or_else(|| anyhow::anyhow!("Missing operator"))?;

        let field = cond[..idx].trim();
        let value = cond[idx + op.len()..].trim();

        if !FIELDS.contains(&field) {
            anyhow::bail!("Invalid field: {}", field);
        }

        if value.is_empty() {
            anyhow::bail!("Missing value for {}", field);
        }

        if i > 0 {
            out.push_str(" AND ");
        }

        out.push_str(field);
        out.push(' ');
        out.push_str(op);
        out.push_str(" ?");
    }

    Ok(out)
}