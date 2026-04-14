use std::{collections::HashSet, f64::NAN, fs::File, io::Write, sync::OnceLock};

mod types;
use types::Files;
pub use types::Record;
pub use types::WeightRecord;
pub use types::PlateRecord;
pub use types::OrderRecord;
pub use types::LogLevel;

static HANDLE: OnceLock<crossbeam::channel::Sender<Record>> = OnceLock::new();

pub fn record_weight(record: WeightRecord) {
    let handle = HANDLE.get().expect("Failed to retrieve handle");
    handle.send(Record::Weight(record)).expect("Why channel full??");
}

pub fn record_plate(record: PlateRecord) {
    let handle = HANDLE.get().expect("Failed to retrieve handle");
    handle.send(Record::Plate(record)).expect("Why channel full??");
}

pub fn record_order(record: Option<OrderRecord>) {
    let handle = HANDLE.get().expect("Failed to retrieve handle");
    handle.send(Record::Order(record)).expect("Why channel full??");
}

pub fn log(level: LogLevel, message: String) {
    let handle = HANDLE.get().expect("Failed to retrieve handle");
    handle.send(Record::Log(level, message)).expect("Why channel full??");
}

pub fn init() {
    let (tx, rx) = crossbeam::channel::unbounded();

    let now = chrono::Local::now();
    let sub_path = now.format("%Y%m%d%H%M%S");

    let files = Files::new(&format!("{}", sub_path));

    std::thread::spawn(move || {
        execute_worker(files, rx);
    });

    HANDLE.set(tx).expect("Failed to init telemetry");
}

fn execute_worker(
    mut files: Files,
    rx_record: crossbeam::channel::Receiver<Record>,
) {
    let mut registered_orders = HashSet::<i32>::new();

    loop {
        let record = rx_record.recv().expect("Channels should exit for the lifetime of the program");

        match record {
            Record::Weight(record)  => write_weight(&mut files.weights, record),
            Record::Plate(record)   => write_plate(&mut files.plates, record),
            Record::Log(level, msg) => write_log(&mut files.logs, level, msg),
            Record::Order(record) => {
                match record {
                    Some(order) => {
                        if registered_orders.contains(&order.id) {
                            continue;
                        }
                        registered_orders.insert(order.id);
                        write_order(&mut files.orders, order);
                    }
                    None => {
                        let order = OrderRecord {
                            id:             0,
                            weight_min:     NAN,
                            weight_max:     NAN,
                            weight_desired: NAN,
                            weight_trigger: NAN,
                        };

                        write_order(&mut files.orders, order);
                    },
                }
            },
        }
    }
}

fn get_timestamp() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

fn write_weight(file: &mut File, record: WeightRecord) {
    let data = format!(
        "{}, {}, {}, {}\n", 
        get_timestamp(), 
        record.weight_0,
        record.weight_1,
        record.weight_total, 
    );

    write_data(file, data);
}

fn write_plate(file: &mut File, record: PlateRecord) {
    let data = format!(
        "{}, {}, {}, {}, {}\n", 
        get_timestamp(), 
        record.peak,
        record.drop,
        record.exit,
        record.in_bounds,
    );

    write_data(file, data);
}

fn write_order(file: &mut File, record: OrderRecord) {
    let data = format!(
        "{}, {}, {}, {}, {}, {}\n", 
        get_timestamp(), 
        record.id,
        record.weight_min,
        record.weight_max,
        record.weight_desired,
        record.weight_trigger,
    );

    write_data(file, data);
}

fn write_log(file: &mut File, level: LogLevel, message: String) {
    let data = format!(
        "{}, {}, {}\n", 
        get_timestamp(), 
        level,
        message,
    );

    write_data(file, data);
}

fn write_data(file: &mut File, data: String) {
    file.write_all(data.as_bytes()).expect("Failed to write");
}