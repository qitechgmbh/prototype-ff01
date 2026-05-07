use std::{collections::HashMap, io::Read, net::{TcpListener, TcpStream}, thread};

use httparse::{EMPTY_HEADER, Request, Status};

mod utils;
mod http;

mod sql;
mod args;
use args::QueryArgs;

mod weights;
mod plates;
mod orders;
mod logs;

mod responses;
use responses::MISSING_PATH;
use responses::UNSUPPORTED_ROUTE;
use responses::INVALID_METHOD;
use responses::MISSING_METHOD;

use crate::query::{sql::FieldType, utils::send_response};

pub fn run(port: u16, db_path: String) -> anyhow::Result<()> {
    let address = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let db_path = db_path.clone();
                thread::spawn(move || handle_client(stream, db_path));
            }
            Err(e) => eprintln!("Failed to accept client: {e}"),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, db_path: String) -> anyhow::Result<()> {
    let mut buf = [0u8; 4096];

    loop {
        let size = stream.read(&mut buf)?;

        if size == 0 {
            // connection closed
            return Ok(());
        }

        let mut headers = [EMPTY_HEADER; 32];
        let mut request = Request::new(&mut headers);

        match request.parse(&buf[..size])? {
            Status::Complete(_) => {
                let Some(method) = request.method else {
                    http::bad_request(&mut stream, MISSING_METHOD)?;
                    continue;
                };

                if method != "GET" {
                    http::bad_request(&mut stream, INVALID_METHOD)?;
                    continue;
                }

                let Some(path) = request.path else {
                    http::bad_request(&mut stream, MISSING_PATH)?;
                    continue;
                };

                let (route, query) = utils::destruct_path(path);

                let args = match QueryArgs::new(query.unwrap_or("")) {
                    Ok(v) => v,
                    Err(e) => {
                        http::bad_request(&mut stream, &e.to_string())?;
                        continue;
                    }
                };
                
                let result = match route {
                    "/weights" => weights::create_sql(args),
                    "/plates"  => plates::create_sql(args),
                    "/orders"  => orders::create_sql(args),
                    "/logs"    => logs::create_sql(args),
                    _ => {
                        http::bad_request(&mut stream, UNSUPPORTED_ROUTE)?;
                        continue;
                    }
                };

                let (sql, params) = match result {
                    Ok(v) => v,
                    Err(e) => {
                        http::internal_error(&mut stream, &e.to_string())?;
                        continue;
                    },
                };

                if let Err(e) = send_response(&db_path, &mut stream, sql, params) {
                    http::internal_error(&mut stream, &e.to_string())?;
                    continue;
                }
            }

            Status::Partial => continue
        }
    }
}