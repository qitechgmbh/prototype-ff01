use std::{env, io::Write, os::unix::net::UnixStream, str::FromStr, thread, time::Duration};

use chrono::Utc;
use rand::{Rng, rngs::ThreadRng};
use telemetry_core::{Event, EventKind, LogCategory, LogEvent, OrderEvent, PlateEvent, WeightBounds, WeightEvent};

fn main() -> std::io::Result<()> {
    let socket_path = env::var("SOCKET_PATH").unwrap();
    
    let mut stream = UnixStream::connect(socket_path)?;

    let mut i = 0;
    loop {
        let mut rng = rand::thread_rng();

        send_weights(&mut rng, &mut stream);
        send_plates(&mut rng, &mut stream);
        send_order(&mut rng, &mut stream, i);
        send_log(&mut rng, &mut stream);
        i += 1;

        thread::sleep(Duration::from_secs_f64(1.0 / 1024.0));
    }
}

fn send_weights(rng: &mut ThreadRng, stream: &mut UnixStream) {
    let event = Event {
        datetime: Utc::now(),
        kind: EventKind::Weight(WeightEvent {
            order_id: None,
            weight_0: Some(rng.gen_range(-500..500)),
            weight_1: Some(rng.gen_range(-500..500)),
        }),
    };

    send_event(event, stream);
}

fn send_plates(rng: &mut ThreadRng, stream: &mut UnixStream) {
    let event = Event {
        datetime: Utc::now(),
        kind: EventKind::Plate(PlateEvent {
            order_id: None,
            peak: rng.gen_range(-500..500),
            real: rng.gen_range(-500..500),
        }),
    };

    send_event(event, stream);
}

fn send_order(rng: &mut ThreadRng, stream: &mut UnixStream, i: u32) {
    let order_id = i;

    let start_event = Event {
        datetime: Utc::now(),
        kind: EventKind::Order(OrderEvent::Started { 
            order_id, 
            worker_id: Some(rng.gen_range(1000..2000), ), 
            bounds: Some(WeightBounds {
                min:     10,
                max:     20,
                desired: 30,
                trigger: 25,
            }) 
        }),
    };

    send_event(start_event, stream);

    let finish_event = if rng.gen_bool(0.5) {
        Event {
            datetime: Utc::now(),
            kind: EventKind::Order(OrderEvent::Completed { 
                order_id, 
                quantity_good:  rng.gen_range(200..350), 
                quantity_scrap: rng.gen_range(0..10), 
            })
        }
    } else {
        Event {
            datetime: Utc::now(),
            kind: EventKind::Order(OrderEvent::Aborted { 
                order_id, 
            })
        }
    };

    send_event(finish_event, stream);
}

fn send_log(rng: &mut ThreadRng, stream: &mut UnixStream) {
    let category: LogCategory = match rng.gen_range(0..3) {
        0 => LogCategory::Debug,
        1 => LogCategory::Info,
        2 => LogCategory::Warn,
        3 => LogCategory::Error,
        _ => unreachable!(),
    };

    let message = heapless::String::from_str("Hello World").unwrap();
    
    let event = Event {
        datetime: Utc::now(),
        kind: EventKind::Log(LogEvent {
            category,
            message,
        }),
    };

    send_event(event, stream);
}

fn send_event(event: Event, stream: &mut UnixStream) {
    println!("Sending: {:?}", &event);

    let mut buf = [0u8; 512];
    
    let data = event.encode(&mut buf[2..]);
    let len  = data.len() as u16;
    buf[0..2].copy_from_slice(&len.to_le_bytes());

    stream.write_all(&buf[0..len as usize + 2]).unwrap();
    stream.flush().unwrap();
}