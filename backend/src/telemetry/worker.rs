use std::{fs::File, path::PathBuf, ptr::null, thread, time::{Duration, SystemTime}};

use crossbeam::channel::{Receiver, TryRecvError};

use crate::telemetry::{RecordType, archiving::save_batch, binary::Batch, request::RecordRequest, types::Files};

pub fn run(exe_dir: PathBuf, root_dir: PathBuf, rx: Receiver<RecordRequest>) {
    let home = dirs::home_dir().expect("no home directory");

    let archive_dir = home.join("telemetry");
    std::fs::create_dir_all(&archive_dir).unwrap();

    let mut batch = Box::new(Batch::new(SystemTime::now(), None));

    loop {
        let now = SystemTime::now();

        // replace batch every 5 minutes
        if now.duration_since(batch.created()).unwrap() >= Duration::from_secs(60 * 5) {
            let old_batch = std::mem::replace(
                &mut batch,
                Box::new(Batch::new(now, None)),
            );

            let dir = archive_dir.clone();

            thread::spawn(move || {
                let batch = old_batch;
                save_batch(dir, &batch);
            });
        }

        loop {
            match rx.try_recv() {
                Ok(request) => handle_request(request, now, &mut *batch),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    panic!("MainThread Channel Disconnected");
                }
            }
        }

        // live request
        // batch request
        // metadata request



        thread::sleep(Duration::from_millis(50));
    }
}

fn handle_request(request: RecordRequest, now: SystemTime, batch: &mut Batch, ) {
    use RecordRequest::*;

    match request {
        Weight { w0, w1 } => batch.append_weight(now, w0, w1),
        Plate { peak, avg, exit } => todo!(),
        Bounds { min, max, desired, trigger } => todo!(),
        State { order_id, state_id } => todo!(),
        Order(record_order_request) => todo!(),
        Log(log_level, _) => todo!(),
    }
}