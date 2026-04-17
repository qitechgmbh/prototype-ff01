use std::{fs::File, io::Write, path::PathBuf};

use chrono::Timelike;
use crossbeam::channel::Receiver;

use crate::telemetry::{Payload, RecordType, types::Files};

pub fn run(exe_dir: PathBuf, root_dir: PathBuf, rx: Receiver<Payload>) {
    let tmp_dir = exe_dir.join("tmp-telemetry");

    let mut last_date = chrono::Local::now();
    let mut files     = Some(Files::new(&tmp_dir));

    loop {
        let (r_type, data) = rx.recv()
            .expect("Channels should exist for the lifetime of the program");

        let current_date = chrono::Local::now();

        let snapshot_complete = 
            last_date.date_naive() != current_date.date_naive() 
            || last_date.hour() != current_date.hour();

        last_date = current_date;

        if snapshot_complete {
            // date changed, create new entry
            let snapshot_id = format!("{:02}", current_date.hour());
            let datestamp   = current_date.format("%Y%m%d").to_string();
            let archive_dir = PathBuf::from(root_dir.join(datestamp).join(snapshot_id));

            // drop files to avoid potential problems when moving the files
            _ = files.take(); // drop current files
            submit_entry(&tmp_dir, &archive_dir);
            files = Some(Files::new(&tmp_dir));
        }

        let files = &mut files.as_mut().expect("Should be None only during transfers");
        let file  = select_file(r_type, files);
        
        file.write_all(data.as_bytes()).expect("Failed to write");
    }
}

fn select_file(r_type: RecordType, files: &mut Files) -> &mut File {
    use RecordType::*;
    match r_type {
        Weight => &mut files.weights,
        Plate  => &mut files.plates,
        State  => &mut files.states,
        Bounds => &mut files.bounds,
        Order  => &mut files.orders,
        Log    => &mut files.logs,
    }
}

/// move measurements from temp to archive dir
fn submit_entry(tmp_dir: &PathBuf, archive_dir: &PathBuf) {
    use std::fs::{create_dir_all};
    create_dir_all(archive_dir).expect("create archive dir failed");
    rename(tmp_dir, archive_dir, "weights.csv");
    rename(tmp_dir, archive_dir, "plates.csv");
    rename(tmp_dir, archive_dir, "states.csv");
    rename(tmp_dir, archive_dir, "bounds.csv");
    rename(tmp_dir, archive_dir, "orders.csv");
    rename(tmp_dir, archive_dir, "logs.csv");
}

fn rename(tmp_dir: &PathBuf, archive_dir: &PathBuf, sub_path: &str) {
    use std::fs;
    let from = tmp_dir.join(sub_path);
    let to   = archive_dir.join(sub_path);
    let err  = &format!("Failed moving {}", sub_path);
    fs::rename(from, to).expect(err);
}