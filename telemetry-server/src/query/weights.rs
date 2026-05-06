use std::{io::Write, net::TcpStream, str::SplitWhitespace};

use arrow::ipc::writer::StreamWriter;
use duckdb::Connection;

#[derive(Debug, Default)]
struct Request {
    from:     Option<u64>,
    to:       Option<u64>,
    order_id: Option<u32>,
}

pub fn process(
    parts: SplitWhitespace<'_>, 
    connection: &Connection,
    stream: &mut TcpStream
) -> anyhow::Result<()> {
    let Some(request) = parse(parts) else {
        const MESSAGE: &str = "Malformed Request";
        stream.write_all(MESSAGE.as_bytes())?;
        return Ok(());
    };

    let sql = build_sql(request);
    let mut statement = connection.prepare(&sql)?;
    
    let mut it  = statement.query_arrow([])?;
    let mut buf = Vec::new();

    // Initialize once with schema from first batch
    if let Some(first_batch) = it.next() {
        let mut writer = StreamWriter::try_new(&mut buf, &first_batch.schema())?;

        // write all batches into buffer
        writer.write(&first_batch)?;
        while let Some(batch) = it.next() {
            writer.write(&batch)?;
        }

        writer.finish()?;
    }

    let len = buf.len() as u64;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&buf)?;

    Ok(())
}

fn build_sql(request: Request) -> String {
    let mut sql = format!("SELECT * FROM weights");

    if let Some(order_id) = request.order_id {
        sql.push_str(&format!(" WHERE order_id: {}", order_id));
    }

    match (request.from, request.to) {
        (Some(from), Some(to)) => {
            let limit = to.saturating_sub(from);
            let data  = format!(" LIMIT {limit} OFFSET {from}");
            sql.push_str(&data);
        }
        (Some(from), None) => {
            let data = format!(" OFFSET {from}");
            sql.push_str(&data);
        }
        (None, Some(to)) => {
            let data = format!(" LIMIT {to}");
            sql.push_str(&data);
        }
        (None, None) => {}
    }

    sql
}

/// Accepts: 
/// FROM  <DateTime>
/// TO    <DateTime>
/// ORDER <u32>
fn parse(parts: SplitWhitespace<'_>) -> Option<Request> {
    let mut request = Request::default();

    let tokens: Vec<&str> = parts.collect();

    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];

        match token {
            "FROM" => {
                if let Some(v) = tokens.get(i + 1) {
                    request.from = v.parse().ok();
                }
                i += 2;
            }
            "TO" => {
                if let Some(v) = tokens.get(i + 1) {
                    request.to = v.parse().ok();
                }
                i += 2;
            }
            "ORDER" => {
                if let Some(v) = tokens.get(i + 1) {
                    request.order_id = v.parse().ok();
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    Some(request)
}