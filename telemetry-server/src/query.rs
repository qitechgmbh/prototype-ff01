use std::{io::{self, BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, path::PathBuf, sync::Arc, thread};

use duckdb::{Config, Connection, types::Value};
use arrow::ipc::writer::StreamWriter;

pub fn run(query_port: u16, db_path: String) -> io::Result<()> {
    let db_path = Arc::new(PathBuf::from(db_path));

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

        // recreate with each request to receive newest snapshots
        // TODO: Determine cause of that behaviour
        let config = Config::default().access_mode(duckdb::AccessMode::ReadOnly)?;
        let connection = Connection::open_with_flags(&*db_path, config)?;

        process(request, connection, &mut stream)?;
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
    connection: Connection, 
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

    let mut buffer = Vec::new();

    let mut it = statement.query_arrow(&refs[..])?;

    let mut i = 0;
    let mut r = 0;

    // Initialize once with schema from first batch
    if let Some(first_batch) = it.next() {
        let mut writer = StreamWriter::try_new(&mut buffer, &first_batch.schema())?;
        writer.write(&first_batch)?;

        i = 1;
        r = first_batch.num_rows();
        while let Some(batch) = it.next() {
            writer.write(&batch)?;
            i += 1;
            r += batch.num_rows();
        }

        writer.finish()?;
    }

    println!("[Query] Sending {i} RecordBatches totaling {r} rows");

    let len = buffer.len() as u64;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&buffer)?;

    Ok(())
}