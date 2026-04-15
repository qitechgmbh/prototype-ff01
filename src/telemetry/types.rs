use std::{fs::{self, File, OpenOptions}, path::{Path, PathBuf}, time::Duration};

use beas_bsl::api::{Date, Time};

pub enum RecordType {
    Weight,
    Plate,
    State,
    Bounds,
    Order,
    Log,
}

pub struct WeightRecord {
    pub weight_0:     f64,
    pub weight_1:     f64,
    pub weight_total: f64,
}

pub struct PlateRecord {
    pub peak:       f64,
    pub drop:       f64,
    pub exit:       f64,
    pub in_bounds:  bool,
}

pub struct ServiceStateRecord {
    pub state_id: u32,
    pub order_id: i32,
}

pub struct WeightBoundsRecord {
    pub order_id: i32,
    pub min:      f64,
    pub max:      f64,
    pub desired:  f64,
    pub trigger:  f64,
}

pub struct OrderRecord {
    pub id:             i32,
    pub personel_id:    String,
    pub quantity_scrap: f64,
    pub quantity_good:  f64,
    pub start_date:     Date,
    pub from_time:      Time,
    pub end_date:       Date,
    pub to_time:        Time,
    pub duration:       Duration,
}

pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info  => write!(f, "INFO"),
            LogLevel::Warn  => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

pub struct Files {
    pub weights: File,
    pub plates:  File,
    pub states:  File,
    pub bounds:  File,
    pub orders:  File,
    pub logs:    File,
}

impl Files {
    pub fn new(sub_path: &str) -> Self {
        let folder_path = format!("/home/qitech/measurements/{}", sub_path);
        let path = Path::new(&folder_path);

        // Recursive directory creation
        fs::create_dir_all(&path).expect("Failed to create folder recursively");

        // Create CSV files inside the folder
        let weights = Self::open_file(path.join("weights.csv"));
        let plates  = Self::open_file(path.join("plates.csv"));
        let states  = Self::open_file(path.join("states.csv"));
        let bounds  = Self::open_file(path.join("bounds.csv"));
        let orders  = Self::open_file(path.join("orders.csv"));
        let logs    = Self::open_file(path.join("logs.csv"));

        return Files { weights, plates, states, bounds, orders, logs }
    }

    fn open_file(path: PathBuf) -> File {
        return OpenOptions::new()
            .create(true)   // create if missing
            .append(true)   // open in append mode
            .open(path)
            .expect("Failed to file!");
    }
}