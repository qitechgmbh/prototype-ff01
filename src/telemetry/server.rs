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
                telemetry::log(LogLevel::Error, format!("[Server]: Opening Stream {}", e));
            }
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, archive_dir: PathBuf) {
    telemetry::log(LogLevel::Info, format!("[Server::Client]: connected"));

    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(len) => {
            let request = &buffer[0..len];

            if request.starts_with(b"GET ") {
                let id_bytes = &request[4..];

                // trim whitespace + CRLF
                let id_str = std::str::from_utf8(id_bytes)
                    .unwrap_or("")
                    .trim() // removes \r, \n, spaces, tabs
                    .trim_end_matches(|c| c == '\r' || c == '\n');

                let Ok(id) = id_str.parse::<u64>()else {
                    let _ = stream.write_all(b"Invalid id for GET request");
                    return;
                };

                let path = archive_dir.join(format!("{}", id).to_string());

                if !path.exists() {
                    let _ = stream.write_all(b"null");
                    return;
                }

                match create_zip(&path) {
                    Ok(zip_bytes) => {
                        let _ = stream.write_all(zip_bytes.as_slice());
                    }
                    Err(e) => {
                        telemetry::log(
                            LogLevel::Error, 
                            format!("[Server::Client]: Zip failed {}", e)
                        );
                        let _ = stream.write_all(b"ZIP_ERROR");
                    }
                }
            } else if request.starts_with(b"LIST ALL") {
                let mut output = String::new();

                if let Ok(entries) = std::fs::read_dir(&archive_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            output.push_str(name);
                            output.push('\n');
                        }
                    }
                }

                let _ = stream.write_all(output.as_bytes());
            } else {
                let _ = stream.write_all(b"Unknown Request");
            }
        }
        Err(e) => {
            telemetry::log(
                LogLevel::Error, 
                format!("[Server::Client]:Failed to read request: {}", e)
            );
            return;
        }
    }
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
