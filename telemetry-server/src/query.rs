use std::{io::{self, BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, path::PathBuf, sync::Arc, thread};

use duckdb::{Config, Connection, types::Value};

pub fn run(query_port: u16, db_path: PathBuf) -> io::Result<()> {
    let db_path = Arc::new(db_path);

    let listener = TcpListener::bind(("0.0.0.0", query_port))?;
    println!("TCP server running on port {}", query_port);

    for stream in listener.incoming() {
        let stream  = stream?;
        let db_path = db_path.clone();

        thread::spawn(move || {
            if let Err(e) = handle_client(stream, db_path) {
                eprintln!("client error: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream, 
    db_path: Arc<PathBuf>
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let config = Config::default().access_mode(duckdb::AccessMode::ReadOnly)?;
    let mut connection = Connection::open_with_flags(&*db_path, config)?;

    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        let request = line.trim();

        if request.is_empty() {
            line.clear();
            continue;
        }

        let Some(request) = parse_request(&line) else {
            continue;
        };

        process(request, &mut connection, &mut stream)?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Request {
    from:     Option<u64>,
    to:       Option<u64>,
    order_id: Option<u64>,
    resource: Resource,
}

#[derive(Debug, Clone, Copy)]
enum Resource {
    Weights
}

fn parse_request(data: &str) -> Option<Request> {
    let mut parts = data.split_whitespace();

    let resource = match parts.next()? {
        "weights" => Resource::Weights,
        _ => return None,
    };

    let mut from     = None;
    let mut to       = None;
    let mut order_id = None;

    // remaining tokens: "FROM", "100", "TO", "200", etc.
    let tokens: Vec<&str> = parts.collect();

    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];

        match token {
            "FROM" => {
                if let Some(v) = tokens.get(i + 1) {
                    from = v.parse().ok();
                }
                i += 2;
            }
            "TO" => {
                if let Some(v) = tokens.get(i + 1) {
                    to = v.parse().ok();
                }
                i += 2;
            }
            "ORDER_ID" => {
                if let Some(v) = tokens.get(i + 1) {
                    order_id = v.parse().ok();
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    Some(Request {
        from,
        to,
        order_id,
        resource,
    })
}

fn process(
    request: Request,
    connection: &mut Connection, 
    stream: &mut TcpStream,
) -> anyhow::Result<()> {
    let table_name = match request.resource {
        Resource::Weights => "weights",
    };

    let mut sql = format!("SELECT * FROM {}", table_name);

    let mut conditions = Vec::new();
    let mut bind_values: Vec<Value> = Vec::new();

    // filters
    if let Some(order_id) = request.order_id {
        conditions.push("order_id = ?");
        bind_values.push(Value::from(order_id));
    }

    // WHERE
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // ORDER
    sql.push_str(" ORDER BY timestamp");

    // pagination
    match (request.from, request.to) {
        (Some(from), Some(to)) => {
            let limit = to.saturating_sub(from);
            sql.push_str(" LIMIT ? OFFSET ?");
            bind_values.push(Value::from(limit));
            bind_values.push(Value::from(from));
        }
        (Some(from), None) => {
            sql.push_str(" OFFSET ?");
            bind_values.push(Value::from(from));
        }
        (None, Some(to)) => {
            sql.push_str(" LIMIT ?");
            bind_values.push(Value::from(to));
        }
        (None, None) => {}
    }

    let mut statement = connection.prepare(&sql)?;

    let refs: Vec<&dyn duckdb::ToSql> =
        bind_values.iter().map(|v| v as &dyn duckdb::ToSql).collect();

    let mut rows = statement.query(&refs[..])?;

    while let Some(row) = rows.next()? {
        let timestamp: i64 = row.get(0)?;
        let order_id:  Option<u32> = row.get(1)?;
        let weight_0:  Option<i16> = row.get(2)?;
        let weight_1:  Option<i16> = row.get(3)?;

        let mut buf = Vec::with_capacity(32);
        let mut flags: u8 = 0;

        // ---- build null bitmask ----
        if order_id.is_none() { flags |= 1 << 0; }
        if weight_0.is_none() { flags |= 1 << 1; }
        if weight_1.is_none() { flags |= 1 << 2; }

        // write data, don't write if field is null
        buf.push(flags);
        buf.extend_from_slice(&timestamp.to_le_bytes());

        if let Some(v) = order_id {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        if let Some(v) = weight_0 {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        if let Some(v) = weight_1 {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        stream.write_all(&buf)?;
    }

    Ok(())
}