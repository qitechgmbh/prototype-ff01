use std::{time::{Duration, Instant}};

mod logging;
use chrono::Date;
use logging::Logger;

mod xtrem;
mod scales;

mod service;
use service::Service;

mod plate_detect_task;
use plate_detect_task::PlateDetectTask;

use crate::scales::Scales;

use std::sync::{Mutex, LazyLock};

static LOGGER: LazyLock<Mutex<Logger>> = LazyLock::new(|| {
    Mutex::new(Logger::new())
});

fn main() {
    // config
    let service_config = service::Config {
        config_path: "/home/qitech/config.json".to_string(),
        reconnect_attempts_max: 10,
        timeout_reconnect: Duration::from_millis(2500),
        timeout_heartbeat: Duration::from_millis(10_000),
        timeout_sending:   Duration::from_millis(100),
    };

    // state
    let mut plate_count: u32 = 0;
    let mut last_print_ts = Instant::now();
    let then = Instant::now();

    // components
    let mut scales  = Scales::new();
    let mut service = Service::new(service_config);
    let mut task: Option<PlateDetectTask> = None;

    // init
    service.set_enabled(true);

    // start
    loop {
        std::thread::sleep(Duration::from_secs_f64(1.0 / 12.0));

        let mut logger = LOGGER.lock().unwrap();

        let now = Instant::now();

        let time = now.duration_since(then).as_millis();

        scales.update();

        let w0 = opt_f64_to_string(scales.weight_0());
        let w1 = opt_f64_to_string(scales.weight_1());
        let wt = opt_f64_to_string(scales.weight_total());

        // Write to file
        logger.log_scales(&format!("[{}], {}, {}, {}", time, w0, w1, wt));

        if now.duration_since(last_print_ts) > Duration::from_millis(1000) {
            let (trigger, peak) = match &task {
                Some(value) => (value.trigger, value.peak.unwrap_or(0.0)),
                None => (0.0, 0.0),
            };

            let chrono_now = chrono::Local::now();

            println!(
                "{} :: weight: ({} + {} = {}) | (task: {} | {}) : (plates: {}) : (ss_id: {})", 
                chrono_now, w0, w1, wt, trigger, peak, plate_count, service.state().index()
            );

            last_print_ts = now;
        }

        if let Err(e) = service.update(now, plate_count) {
            println!("Error while updating service: {}", e);
        }
        
        let entry = match service.state() {
            service::State::Zero => {
                task        = None;
                plate_count = 0;
                logger.set_order(None);
                continue;
            },
            service::State::One(state) => &state.entry,
            service::State::Two(_) => {
                task = None;
                logger.set_order(None);
                continue;
            },
        };

        // init task 
        if task.is_none() {
            let min     = entry.weight_bounds.min;
            let trigger = min * 0.8;
            task = Some(PlateDetectTask::new(trigger));

            logger.set_order(Some(entry.doc_entry));
        }

        let task = task.as_mut().unwrap();

        let weight_total = scales.weight_total().expect("Scales disconnected!");

        if let Some((plate, drop)) = task.check(scales.weight_total().unwrap()) {
            let in_bounds = 
                entry.weight_bounds.min <= weight_total 
                && weight_total <= entry.weight_bounds.max;

            let msg = format!(
                "[{}], {}, {}, {}, {}, {}", 
                time, 
                plate,
                drop,                         
                entry.weight_bounds.min,
                entry.weight_bounds.max, 
                in_bounds
            );

            logger.log_task(&msg);
            plate_count += 1;
        }
    }
}

fn opt_f64_to_string(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1}", x),
        None => "_._".to_string(), // or "NaN"
    }
}