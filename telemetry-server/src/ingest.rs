use crossbeam::channel::TrySendError;
use std::{
    fs,
    io::{self, Read},
    os::unix::net::UnixListener,
    sync::Arc,
};

use crate::{EVENT_LEN_SIZE, EVENT_SIZE_MAX, PayloadSender, SUBSCRIBER_COUNT};
use telemetry_core::Event;

pub fn run(
    socket_path: String, 
    subscribers: Arc<[PayloadSender; SUBSCRIBER_COUNT]>
) -> anyhow::Result<()> {
    use io::ErrorKind::NotFound;
    use io::ErrorKind::UnexpectedEof;

    if let Err(e) = fs::remove_file(&socket_path) {
        if e.kind() != NotFound {
            return Err(e.into());
        }
    }

    let listener = UnixListener::bind(socket_path)?;
    let mut buf = [0u8; EVENT_SIZE_MAX];

    loop {
        // wait for next connection. Only supports one unix socket connection at a time
        let (mut stream, _) = listener.accept()?;

        loop {
            if let Err(e) = stream.read_exact(&mut buf[..EVENT_LEN_SIZE]) {
                if e.kind() != UnexpectedEof {
                    eprintln!("[Ingest] Error while reading from stream {}", e);
                }
                break;
            }

            let len = u16::from_le_bytes(buf[..EVENT_LEN_SIZE].try_into().unwrap()) as usize;

            if let Err(e) = stream.read_exact(&mut buf[..len]) {
                if e.kind() != UnexpectedEof {
                    eprintln!("[Ingest] Error while reading from stream {}", e);
                }
                break;
            }

            let data = &buf[..len];

            // if data is malformed we are out of sync with the stream
            if Event::decode(data).is_none() {
                eprintln!("[Ingest] Received malformed data. Discarding connection!");
                break;
            };

            let message: Arc<Vec<u8>> = Arc::new(data.to_vec());

            // forward message to all subscribers
            for subscriber in subscribers.as_slice() {
                if let Err(TrySendError::Full(_)) = subscriber.try_send(message.clone()) {
                    eprintln!("[Ingest] Failed to send: Channel Full!");
                };
            }
        }
    }
}
