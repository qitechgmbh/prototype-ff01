use std::env;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;

use arc_swap::ArcSwap;
use crossbeam::channel::Sender;

use crate::telemetry::binary::Batch;

mod tcp_handler;
mod request;
mod archiving;
mod binary;
mod worker;

type Payload = (RecordType, String);

static HANDLE: OnceLock<(Sender<Payload>, Sender<Payload>)> = OnceLock::new();

#[derive(Debug)]
pub struct Shared {
    pub batch_id_current: AtomicU64,
    pub batch_snapshot: ArcSwap<Batch>,
}

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
        "{}, {:.1}, {:.1}, {:.1}\n", 
        get_timestamp(), 
        record.peak,
        record.drop,
        record.exit,
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
    let (worker_tx, live_data_tx) = HANDLE.get().expect("Failed to retrieve handle");
    worker_tx.send((r_type, data.clone())).expect("worker channel is full");
    live_data_tx.send((r_type, data)).expect("live data channel is full");
}

#[allow(deprecated)]
pub fn init() {
    let (tx0, rx0) = crossbeam::channel::unbounded();
    let (tx1, rx1) = crossbeam::channel::unbounded();

    let exe_path: PathBuf = env::current_exe().expect("Requires exe path");
    let exe_dir = exe_path
        .parent()
        .expect("Executable must have a parent directory")
        .to_path_buf();

    let home_dir         = std::env::home_dir().expect("Needs home dir");
    let archive_root_dir = home_dir.join("telemetry");

    HANDLE.set((tx0, tx1)).expect("Failed to init telemetry");

    std::thread::spawn({
        let archive_root_dir = archive_root_dir.clone();
        move || worker::run(exe_dir, archive_root_dir, rx0)
    });

    std::thread::spawn({
        let archive_root_dir = archive_root_dir.clone();
        move || server::run(archive_root_dir)
    });

    std::thread::spawn(move || live_data::run(rx1));
}



fn get_timestamp() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%dT%H:%M:%S%.6f").to_string()
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    #[test]
    pub fn my_test() {
        super::init();
        loop {
            // seed from current time
            let mut seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();

            let w0 = (next_rand(&mut seed) % 6) as f64;
            let w1 = (next_rand(&mut seed) % 6) as f64;

            super::record_weight(super::WeightRecord { 
                weight_0: w0, 
                weight_1: w1, 
                weight_total: w0 + w1 
            });

            //super::log(LogLevel::Info, "Hello World".to_string());
            std::thread::sleep(Duration::from_millis(1000 / 12));
        }
    }

    fn next_rand(seed: &mut u32) -> u32 {
        let mut x = *seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *seed = x;
        x
    }
}