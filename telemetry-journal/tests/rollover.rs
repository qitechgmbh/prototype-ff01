use std::{fs, io, path::PathBuf, thread, time::Duration};

use chrono::{Local, TimeZone};
use crossbeam::channel;
use telemetry_core::{Event, LogCategory, LogEvent, OrderEvent, PlateEvent, Entry, WeightEvent};

#[test]
fn test_rollover() -> io::Result<()> {
    const OFFSET_PRE:  u32 = 2;
    const OFFSET_POST: u32 = 2;

    let root_dir: PathBuf = "sandbox/telemetry".into();
    fs::remove_dir_all(&root_dir)?;

    let dt_base = Local
        .with_ymd_and_hms(2026, 4, 20, 23, 59, 60 - OFFSET_PRE)
        .unwrap();

    let journal_config = telemetry_journal::Config {
        dir_logs:       root_dir.join("logs"),
        dir_archive:    root_dir.join("archive"),
        flush_interval: Duration::from_micros(1000),
        initial_date:   Some(dt_base),
    };

    let (tx, rx) = channel::bounded(512);
    
    _ = thread::spawn(move || {
        if let Err(e) = telemetry_journal::run(journal_config, rx) {
            println!("Thread exited with status: {:?}", e);
        }
    });

    for i in 0..OFFSET_PRE + OFFSET_POST {
        let dt_next = dt_base + Duration::from_secs(i as u64);

        println!("now: {}", dt_next);
        let ts = dt_next.timestamp_micros() as u64;

        let entry = Entry {
            timestamp: ts,
            event: Event::Weight(WeightEvent {
                weight_0: Some(100),
                weight_1: Some(200),
            }),
        };
        _ = tx.send(entry);

        let entry = Entry {
            timestamp: ts,
            event: Event::Plate(PlateEvent {
                peak: 100,
                real: 90,
            }),
        };
        _ = tx.send(entry);

        let entry = Entry {
            timestamp: ts,
            event: Event::Order(OrderEvent {
                started:     false,
                order_id:    12345,
                worker_id:   None,
                scrap_count: None,
            }),
        };
        _ = tx.send(entry);

        let entry = Entry {
            timestamp: ts,
            event: Event::Log(LogEvent {
                category: LogCategory::Debug,
                message:  "Hello World!".into(),
            }),
        };
        _ = tx.send(entry);

        thread::sleep(Duration::from_millis(1000));
    }

    thread::sleep(Duration::from_millis(100));

    Ok(())
}