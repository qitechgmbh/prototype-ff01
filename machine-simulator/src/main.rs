use std::{
    io::Write,
    os::unix::net::UnixStream,
    thread,
    time::{SystemTime, UNIX_EPOCH, Duration},
};

use rand::Rng;
use telemetry_core::{Entry, Event, WeightEvent};

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/qitech_telemetry.sock";
    let mut stream = UnixStream::connect(socket_path)?;

    loop {
        let mut rng = rand::thread_rng();

        let entry = Entry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,

            event: Event::Weight(WeightEvent {
                weight_0: Some(rng.gen_range(-500..500)),
                weight_1: Some(rng.gen_range(-500..500)),
            }),
        };

        println!("Sending: {:?}", &entry);

        let mut buf = [0u8; 1024];
        let payload = postcard::to_slice(&entry, &mut buf).unwrap();

        // u16 length prefix (your protocol)
        let len = payload.len() as u16;
        stream.write_all(&len.to_le_bytes())?;
        stream.write_all(&payload)?;

        stream.flush()?;

        thread::sleep(Duration::from_secs(1));
    }
}