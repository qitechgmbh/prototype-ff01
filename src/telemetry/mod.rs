mod types;
use std::io::Write;
use std::sync::OnceLock;

use crossbeam::channel::Sender;
use crossbeam::channel::Receiver;

use types::Files;
pub use types::WeightRecord;
pub use types::PlateRecord;
pub use types::ServiceStateRecord;
pub use types::WeightBoundsRecord;
pub use types::OrderRecord;
pub use types::LogLevel;
pub use types::RecordType;

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
        "{}, {}, {:.1}, {:.1}, {:.1}, {:.1}\n", 
        get_timestamp(), 
        record.order_id,
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

pub fn init() {
    let (tx, rx) = crossbeam::channel::unbounded();

    let now = chrono::Local::now();
    let sub_path = now.format("%Y%m%d%H%M%S");

    let files = Files::new(&format!("{}", sub_path));

    std::thread::spawn(move || execute_worker(files, rx));

    HANDLE.set(tx).expect("Failed to init telemetry");
}

fn execute_worker(mut files: Files,rx_record: Receiver<Payload>) {
    loop {
        let (r_type, data) = rx_record.recv()
            .expect("Channels should exit for the lifetime of the program");

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

fn get_timestamp() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}