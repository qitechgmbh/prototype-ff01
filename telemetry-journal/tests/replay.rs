use std::{fs::{self, File}, io, path::PathBuf, thread, time::Duration};

use chrono::{Local, TimeZone};
use crossbeam::channel;
use telemetry_journal::{Event, LogCategory, LogEvent, OrderEvent, PlateEvent, WalEntry, WeightEvent, journal};

#[test]
fn test_roundtrip() -> io::Result<()> {
    let root_dir: PathBuf = "sandbox/telemetry".into();
    fs::remove_dir_all(&root_dir)?;

    let dt_base = Local.with_ymd_and_hms(2026, 04, 20, 0, 0, 0).unwrap();

    let journal_config = journal::Config {
        dir_logs:       root_dir.join("logs"),
        dir_archive:    root_dir.join("archive"),
        flush_interval: Duration::from_micros(1000),
        initial_date:   Some(dt_base),
    };

    let (tx, rx) = channel::bounded(512);
    
    _ = thread::spawn(move || {
        if let Err(e) = journal::run(journal_config, rx) {
            println!("Thread exited with status: {:?}", e);
        }
    });

    for i in 0..10 {
        let dt_next = dt_base + Duration::from_millis(i * 50 as u64);

        let ts = dt_next.timestamp_micros() as u64;

        let entry = WalEntry {
            timestamp: ts,
            event: Event::Weight(WeightEvent {
                weight_0: Some(1),
                weight_1: Some(2),
            }),
        };
        _ = tx.send(entry);

        let entry = WalEntry {
            timestamp: ts,
            event: Event::Plate(PlateEvent {
                peak: 100,
                real: 90,
            }),
        };
        _ = tx.send(entry);

        let entry = WalEntry {
            timestamp: ts,
            event: Event::Order(OrderEvent {
                started:     false,
                order_id:    12345,
                worker_id:   None,
                scrap_count: None,
            }),
        };
        _ = tx.send(entry);

        let entry = WalEntry {
            timestamp: ts,
            event: Event::Log(LogEvent {
                category: LogCategory::Debug,
                message:  "Hello World!".into(),
            }),
        };
        _ = tx.send(entry);

        thread::sleep(Duration::from_millis(50));
    }

    let path = PathBuf::from("sandbox/telemetry/logs/20260420.wal");
    let mut file = File::open(path).unwrap();

    loop {
        let entry = match WalEntry::read(&mut file) {
            Ok(opt_v) => match opt_v {
                Some(v) => v,
                None => break,
            },
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    println!("Found corrupted log!");
                    break;
                }

                return Err(e);
            },
        };

        println!("Entry: {:?}", entry);
    }

    Ok(())
}