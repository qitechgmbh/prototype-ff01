use std::{
    io::{self, Write},
    os::unix::net::UnixStream,
    thread,
    time::Duration,
};

use chrono::Utc;
use rand::{Rng, rngs::ThreadRng};
use telemetry_core::{Entry, Event, LogCategory, LogEvent, PlateEvent, WeightEvent};

fn main() -> std::io::Result<()> {
    let socket_path = "/tmp/qitech_telemetry.sock";
    let mut stream = UnixStream::connect(socket_path)?;

    loop {
        let mut rng = rand::thread_rng();

        send_weights(&mut rng, &mut stream)?;
        send_plates(&mut rng, &mut stream)?;
        send_log(&mut rng, &mut stream)?;

        thread::sleep(Duration::from_secs_f64(1.0 / 32.0));
    }
}

fn send_weights(rng: &mut ThreadRng, stream: &mut UnixStream) -> io::Result<()> {
    let entry = Entry {
        timestamp: Utc::now(),

        event: Event::Weight(WeightEvent {
            order_id: None,
            weight_0: Some(rng.gen_range(-500..500)),
            weight_1: Some(rng.gen_range(-500..500)),
        }),
    };

    println!("Sending: {:?}", &entry);

    let mut buf = [0u8; 1024];
    let payload = postcard::to_slice(&entry, &mut buf).unwrap();

    let len = payload.len() as u16;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

fn send_plates(rng: &mut ThreadRng, stream: &mut UnixStream) -> io::Result<()> {
    let entry = Entry {
        timestamp: Utc::now(),
        event: Event::Plate(PlateEvent {
            peak: rng.gen_range(-500..500),
            real: rng.gen_range(-500..500),
        }),
    };

    println!("Sending: {:?}", &entry);

    let mut buf = [0u8; 1024];
    let payload = postcard::to_slice(&entry, &mut buf).unwrap();

    let len = payload.len() as u16;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

fn send_log(rng: &mut ThreadRng, stream: &mut UnixStream) -> io::Result<()> {
    let category: LogCategory = match rng.gen_range(0..3) {
        0 => LogCategory::Debug,
        1 => LogCategory::Info,
        2 => LogCategory::Warn,
        3 => LogCategory::Error,
        _ => unreachable!(),
    };

    let message = "Hello World".to_string();
    
    let entry = Entry {
        timestamp: Utc::now(),
        event: Event::Log(LogEvent {
            category,
            message,
        }),
    };

    println!("Sending: {:?}", &entry);

    let mut buf = [0u8; 1024];
    let payload = postcard::to_slice(&entry, &mut buf).unwrap();

    let len = payload.len() as u16;
    stream.write_all(&len.to_le_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}