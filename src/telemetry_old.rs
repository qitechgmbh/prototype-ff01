use std::{fs::{self, File, OpenOptions}, path::{Path, PathBuf}, sync::OnceLock, time::{SystemTime, UNIX_EPOCH}};
use std::io::Write;

use chrono::{self, Local};

pub static HANDLE: OnceLock<crossbeam::channel::Sender<Entry>> = OnceLock::new();

pub enum Entry {
    Weight(WeightEntry),
    Plate(PlateEntry),
    Event(EventEntry),
    Order(Option<i32>),
}

pub struct WeightEntry {
    pub weight_0:       f64,
    pub weight_1:       f64,
    pub weight_total:   f64,
    pub weight_min:     f64,
    pub weight_max:     f64,
    pub weight_desired: f64,
}

pub struct PlateEntry {
    pub triggger:   f64,
    pub peak:       f64,
    pub drop:       f64,
    pub exit:       f64,
    pub in_bounds:  u32,
}

pub struct OrderEntry {
    pub id:             i32,
    pub status:         f64,
    pub weight_min:     f64,
    pub weight_max:     f64,
    pub weight_desired: f64,
}

pub struct EventEntry {
    pub event_type: EventType,
    pub message:    String,
}

pub enum EventType {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Info  => write!(f, "INFO"),
            EventType::Warn  => write!(f, "WARN"),
            EventType::Error => write!(f, "ERROR"),
        }
    }
}

pub struct Logger {
    global: Entry,
    order:  Option<(i32, Entry)>,
}

pub struct MeasurementEntry {
    weights: File,
    plates:  File,
    events:  File,
}

pub fn start() -> crossbeam::channel::Sender<Entry> {
    let (tx, rx) = crossbeam::channel::unbounded();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let timestamp = now.as_secs();

    let logger = Logger {
        global: init_entry(&format!("any_{}", timestamp)),
        order:  None,
    };

    std::thread::spawn(move || {
        execute_worker(logger, rx);
    });

    return tx;
}

fn init_entry(sub_path: &str) -> Entry {
    let folder_path = format!("/home/qitech/measurements/{}", sub_path);
    let path = Path::new(&folder_path);

    // Recursive directory creation
    fs::create_dir_all(&path).expect("Failed to create folder recursively");

    // Create 3 CSV files inside the folder
    let weights = open_file(path.join("weights.csv"));
    let plates  = open_file(path.join("plates.csv"));
    let events  = open_file(path.join("events.csv"));

    return Entry { weights, plates, events }
}

fn open_file(path: PathBuf) -> File {
    return OpenOptions::new()
        .create(true)   // create if missing
        .append(true)   // open in append mode
        .open(path)
        .expect("Failed to file!");
}

fn execute_worker(
    mut logger: Logger,
    rx_msg: crossbeam::channel::Receiver<Entry>,
) {
    loop {
        let msg_any = rx_msg.recv().expect("Channels should exit for the lifetime of the program");

        // Format timestamp
        let now = Local::now();
        let date = now.format("%d.%m.%Y").to_string();
        let time = now.format("%H:%M:%S").to_string();

        match msg_any {
            Entry::Weight(data) => {
                let out = format!(
                    "{}, {}, {}, {}, {}, {}, {}, {}\n", 
                    date, 
                    time, 
                    data.weight_0,
                    data.weight_1,
                    data.weight_total, 
                    data.weight_min, 
                    data.weight_max,
                    data.weight_desired
                );
                logger.global.weights.write_all(out.as_bytes()).expect("Failed to write");
                if let Some(order) = &mut logger.order {
                    order.1.weights.write(out.as_bytes()).expect("Failed to write");
                }
            },
            Entry::Plate(data) => {
                let out = format!(
                    "{}, {}, {}, {}, {}, {}, {}\n", 
                    date, 
                    time, 
                    data.triggger,
                    data.peak,
                    data.drop,
                    data.exit,
                    data.in_bounds,
                );
                logger.global.plates.write_all(out.as_bytes()).expect("Failed to write");
                if let Some(order) = &mut logger.order {
                    order.1.plates.write(out.as_bytes()).expect("Failed to write");
                }
            },
            Entry::Event(data) => {
                let out = format!(
                    "{}, {}, {}, {}\n", 
                    date, 
                    time, 
                    data.event_type,
                    data.message,
                );
                logger.global.events.write_all(out.as_bytes()).expect("Failed to write");
                if let Some(order) = &mut logger.order {
                    order.1.events.write(out.as_bytes()).expect("Failed to write");
                }
            },
            Entry::Order(order_id) => {
                match order_id {
                    Some(id) => {
                        if let Some(order) = &logger.order {
                            if order.0 != id {
                                let entry = init_entry(&format!("order_{}", id));
                                logger.order = Some((id, entry));
                            }
                        } else {
                            let entry = init_entry(&format!("order_{}", id));
                            logger.order = Some((id, entry));
                        }
                    },
                    None => {
                        logger.order = None;
                    },
                }
            },
        }
    }
}