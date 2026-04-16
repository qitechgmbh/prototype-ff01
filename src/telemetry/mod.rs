use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

use chrono::Timelike;
use crossbeam::channel::Sender;
use crossbeam::channel::Receiver;

mod types;
use types::Files;
pub use types::WeightRecord;
pub use types::PlateRecord;
pub use types::ServiceStateRecord;
pub use types::WeightBoundsRecord;
pub use types::OrderRecord;
pub use types::LogLevel;
pub use types::RecordType;

mod server;

type Payload = (RecordType, String);

static HANDLE: OnceLock<Sender<Payload>> = OnceLock::new();

pub fn record_weight(record: WeightRecord) {
    let data = format!(
        "{}, {:.1}, {:.1}, {:.1}\n", 
        get_timestamp(), 
        record.weight_0,
        record.weight_1,
        record.weight_total, 
    );

    send(RecordType::Weight, data);
}

pub fn record_plate(record: PlateRecord) {
    let data = format!(
        "{}, {:.1}, {:.1}, {:.1}, {}\n", 
        get_timestamp(), 
        record.peak,
        record.drop,
        record.exit,
        record.in_bounds,
    );

    send(RecordType::Plate, data);
}

pub fn record_state(record: ServiceStateRecord) {
    let data = format!(
        "{}, {}, {}\n", 
        get_timestamp(), 
        record.state_id,
        record.order_id,
    );

    send(RecordType::State, data);
}

pub fn record_bounds(record: WeightBoundsRecord) {
    let data = format!(
        "{}, {:.1}, {:.1}, {:.1}, {:.1}\n", 
        get_timestamp(), 
        record.min,
        record.max,
        record.desired,
        record.trigger,
    );

    send(RecordType::Bounds, data);
}

pub fn record_order(record: OrderRecord) {
    let start_time = format!(
        "{}-{}-{}T{}:{}",
        record.start_date.year,
        record.start_date.month,
        record.start_date.day,
        record.from_time.hour,
        record.from_time.minute,
    );

    let end_time = format!(
        "{}-{}-{}T{}:{}",
        record.end_date.year,
        record.end_date.month,
        record.end_date.day,
        record.to_time.hour,
        record.to_time.minute,
    );

    let data = format!(
        "{}, {}, {}, {:.1}, {:.1}, {}, {}, {:.2}\n",
        get_timestamp(),
        record.id,
        record.personel_id,
        record.quantity_scrap,
        record.quantity_good,
        start_time,
        end_time,
        record.duration.as_secs_f64(),
    );

    send(RecordType::Order, data);
}

pub fn log(level: LogLevel, message: String) {
    let data = format!(
        "{}, {}, {}\n", 
        get_timestamp(), 
        level,
        message,
    );

    send(RecordType::Log, data);
}

fn send(r_type: RecordType, data: String) {
    let handle = HANDLE.get().expect("Failed to retrieve handle");
    handle.send((r_type, data)).expect("Why channel full??");
}

#[allow(deprecated)]
pub fn init() {
    let (tx, rx) = crossbeam::channel::unbounded();

    let exe_path: PathBuf = env::current_exe().expect("Requires exe path");
    let exe_dir = exe_path
        .parent()
        .expect("Executable must have a parent directory")
        .to_path_buf();

    let home_dir         = std::env::home_dir().expect("Needs home dir");
    let archive_root_dir = home_dir.join("telemetry");

    std::thread::spawn({
        let archive_root_dir = archive_root_dir.clone();
        move || server::run(archive_root_dir)
    });

    std::thread::spawn({
        let archive_root_dir = archive_root_dir.clone();
        move || execute_worker(exe_dir, archive_root_dir, rx)
    });

    HANDLE.set(tx).expect("Failed to init telemetry");
}

fn execute_worker(exe_dir: PathBuf, root_dir: PathBuf, rx_record: Receiver<Payload>) {
    let tmp_dir = exe_dir.join("tmp-telemetry");

    let mut last_date = chrono::Local::now();
    let mut files     = Some(Files::new(&tmp_dir));

    loop {
        let (r_type, data) = rx_record.recv()
            .expect("Channels should exist for the lifetime of the program");

        let current_date = chrono::Local::now();

        let snapshot_complete = 
            last_date.date_naive() != current_date.date_naive() 
            || last_date.hour() != current_date.hour()
            || last_date.minute() != current_date.minute();

        last_date = current_date;

        if snapshot_complete {
            // date changed, create new entry
            let snapshot_id = current_date.minute().to_string();
            let datestamp   = current_date.format("%Y%m%d").to_string();
            let archive_dir = PathBuf::from(root_dir.join(datestamp).join(snapshot_id));

            // drop files to avoid potential problems when moving the files
            _ = files.take(); // drop current files
            submit_entry(&tmp_dir, &archive_dir);
            files = Some(Files::new(&tmp_dir));
        }

        let files = &mut files.as_mut() .expect("Only None during transfer");
        let file = match r_type {
            RecordType::Weight => &mut files.weights,
            RecordType::Plate  => &mut files.plates,
            RecordType::State  => &mut files.states,
            RecordType::Bounds => &mut files.bounds,
            RecordType::Order  => &mut files.orders,
            RecordType::Log    => &mut files.logs,
        };

        file.write_all(data.as_bytes()).expect("Failed to write");
    }
}

/// move measurements from temp to archive dir
fn submit_entry(tmp_dir: &PathBuf, archive_dir: &PathBuf) {
    std::fs::create_dir_all(archive_dir).expect("create archive dir failed");

    std::fs::rename(tmp_dir.join("weights.csv"), archive_dir.join("weights.csv")).expect("move weights failed");
    std::fs::rename(tmp_dir.join("plates.csv"),  archive_dir.join("plates.csv")).expect("move plates failed");
    std::fs::rename(tmp_dir.join("states.csv"),  archive_dir.join("states.csv")).expect("move states failed");
    std::fs::rename(tmp_dir.join("bounds.csv"),  archive_dir.join("bounds.csv")).expect("move bounds failed");
    std::fs::rename(tmp_dir.join("orders.csv"),  archive_dir.join("orders.csv")).expect("move orders failed");
    std::fs::rename(tmp_dir.join("logs.csv"),    archive_dir.join("logs.csv")).expect("move logs failed");
}

fn get_timestamp() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

#[cfg(test)]
mod test {
    #[test]
    pub fn my_test() {
        super::init();
        loop {}
    }
}