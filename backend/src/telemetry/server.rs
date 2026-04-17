use std::{
    fs::File,
    io::{Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    thread,
};

use zip::{ZipWriter, write::FileOptions};

use crate::telemetry::{self, LogLevel};

pub fn run(archive_dir: PathBuf) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565")?;

    for stream in listener.incoming() {
        let dir = archive_dir.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream, dir));
            }
            Err(e) => {
                let msg = format!("[Server]: Opening Stream {}", e);
                telemetry::log(LogLevel::Error, msg);
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, archive_dir: PathBuf) {
    let msg = format!("[Server::Client]: connected");
    telemetry::log(LogLevel::Info, msg);

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(len) => {
            let request = &buffer[0..len];

            let response = if request.starts_with(b"GET ") {
                if request.len() >= 5 {
                    handle_get_request(&request[4..], &archive_dir)
                } else {
                    Vec::from(b"[Error] Missing id")
                }
            } else if request.starts_with(b"LIST ALL") {
                handle_list_all_request(&archive_dir)
            } else {
                Vec::from(b"[Error] Unknown Request")
            };

            _ = stream.write_all(&response);
        }
        Err(e) => {
            let msg = format!("[Server::Client]:Failed to read request: {}", e);
            telemetry::log(LogLevel::Error, msg);
        }
    }
}

fn handle_get_request(
    data: &[u8], 
    archive_dir: &PathBuf
) -> Vec<u8> {
    // trim whitespace + CRLF
    let id_str = std::str::from_utf8(data)
        .unwrap_or("")
        .trim() // removes \r, \n, spaces, tabs
        .trim_end_matches(|c| c == '\r' || c == '\n');

    let Ok(id) = id_str.parse::<u64>()else {
        return Vec::from(b"[Error] Failed to parse id");
    };

    let path = archive_dir.join(format!("{}", id).to_string());

    if !path.exists() {
        return Vec::from(b"[Error] No such id");
    }

    match create_zip(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            let msg = format!("[Server::Client]: Zip failed {}", e);
            telemetry::log(LogLevel::Error, msg.clone());
            Vec::from(format!("[Error]: Zip failed {}", e))
        }
    }
}

fn handle_list_all_request(archive_dir: &PathBuf) -> Vec<u8> {
    let mut output = String::new();

    if !archive_dir.exists() {
        return Vec::from(b"[Error] Missing archive dir");
    }

    if let Ok(entries) = std::fs::read_dir(&archive_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                output.push_str(name);
                output.push('\n');
            }
        }
    }

    Vec::from(output.as_bytes())
}

fn create_zip(dir: &Path) -> std::io::Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut buffer);

    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let walkdir = walkdir::WalkDir::new(dir);

    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(dir).unwrap();

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;

            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(buffer.into_inner())
}
