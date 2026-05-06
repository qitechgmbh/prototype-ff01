use std::{
    fs,
    io::{self, Read},
    os::unix::net::UnixListener,
    sync::Arc,
};
use crossbeam::channel::TrySendError;

use telemetry_core::Event;
use crate::PayloadSender;

pub fn run(socket_path: String, subscribers: Arc<[PayloadSender; 2]>) -> anyhow::Result<()> {
    use io::ErrorKind::NotFound;
    use io::ErrorKind::UnexpectedEof;

    if let Err(e) = fs::remove_file(&socket_path) {
        if e.kind() != NotFound {
            return Err(e.into());
        }
    }

    let listener = UnixListener::bind(socket_path)?;
    let mut buf = [0u8; 512];

    loop {
        let (mut stream, _) = listener.accept()?;

        loop {
            if let Err(e) = stream.read_exact(&mut buf[0..2]) {
                if e.kind() != UnexpectedEof {
                    eprintln!("[Ingest] Error while reading from stream {}", e);
                }
                break;
            }

            let len = u16::from_le_bytes(buf[0..2].try_into().unwrap()) as usize;

            if let Err(e) = stream.read_exact(&mut buf[..len]) {
                if e.kind() != UnexpectedEof {
                    eprintln!("[Ingest] Error while reading from stream {}", e);
                }
                break;
            }

            let data = &buf[0..len];

            // if data is malformed we are out of sync with the stream
            if Event::decode(data).is_none() {
                eprintln!("[Ingest] Received malformed data. Dropping connection!");
                break;
            };

            println!("[Ingest] Received {:?}", Event::decode(data).unwrap());

            let message: Arc<Vec<u8>> = Arc::new(data.to_vec());

            let mut i = 0;
            // forward message to all subscribers
            for subscriber in subscribers.as_slice() {

                if let Err(TrySendError::Full(_)) = subscriber.try_send(message.clone()) {
                    eprintln!("[Ingest] failed to send: Channel Full!");
                    eprintln!("NUM: {i}");
                };

                i += 1;
            }
        }
    }
}
