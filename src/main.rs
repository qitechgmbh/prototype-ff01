use std::{f64::NAN, time::{Duration, Instant}};

mod telemetry;

mod xtrem;
mod scales;

mod service;
use service::Service;

mod plate_detect_task;
use plate_detect_task::PlateDetectTask;

use crate::{
    scales::Scales, 
    service::State
};

use crate::telemetry::{
    LogLevel, 
    OrderRecord, 
    PlateRecord, 
    WeightRecord
};

pub struct App {
    // state
    pub plate_count: u32,
    pub last_print_ts: Instant,

    // components
    pub scales:  Scales,
    pub service: Service,
    pub task:    Option<PlateDetectTask>,
}

impl App {
    pub fn update(&mut self, now: Instant) {
        self.scales.update();

        if self.print_ready(now) {
            self.print_state();
            self.last_print_ts = now;   
        }

        let (w0, w1, wt, complete) = {
            let mut complete: bool = true;
            if self.scales.weight_0().is_none() {
                telemetry::log(LogLevel::Error, format!("Weight 0 is None!"));
                complete = false;
            }

            if self.scales.weight_1().is_none() {
                telemetry::log(LogLevel::Error, format!("Weight 1 is None!"));
                complete = false;
            }

            let w0 = self.scales.weight_0().unwrap_or(NAN);
            let w1 = self.scales.weight_1().unwrap_or(NAN);

            (w0, w1, w0 + w1, complete)
        };

        if !complete {
            telemetry::record_weight(WeightRecord {
                weight_0:     w0,
                weight_1:     w1,
                weight_total: wt,
            });
            return;
        }

        let state_modified = match self.service.update(now, self.plate_count) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Error while updating service: {}", e);
                println!("{}", msg);
                telemetry::log(LogLevel::Error, msg);
                return;
            },
        };

        if state_modified {
            self.task = None;

            if let State::One(state) = self.service.state() {
                let entry = &state.entry;

                // reset counter
                self.plate_count = 0;

                // init task
                let min     = entry.weight_bounds.min;
                let trigger = min * 0.8;
                self.task = Some(PlateDetectTask::new(trigger));

                // record order
                telemetry::record_order(Some(OrderRecord {
                    id:             entry.doc_entry,
                    weight_min:     entry.weight_bounds.min,
                    weight_max:     entry.weight_bounds.max,
                    weight_desired: entry.weight_bounds.desired,
                    weight_trigger: trigger,
                }));
            }
        }

        let State::One(state) = self.service.state() else {
            return;
        };

        let entry = &state.entry;
        let task  = self.task.as_mut().expect("Initialized when entering state");

        if let Some((peak, drop)) = task.check(wt) {
            let bounds = &entry.weight_bounds;

            let exit = wt;
            let in_bounds = bounds.min <= wt && wt <= bounds.max;

            telemetry::record_plate(PlateRecord { peak, drop, exit, in_bounds });

            self.plate_count += 1;
        }
    }

    fn print_ready(&self, now: Instant) -> bool {
        now.duration_since(self.last_print_ts) > Duration::from_millis(1000)
    }

    fn print_state(&mut self) {
        let (trigger, peak) = match &self.task {
            Some(value) => (value.trigger, value.peak.unwrap_or(0.0)),
            None => (0.0, 0.0),
        };

        let chrono_now = chrono::Local::now();

        let w0 = opt_f64_to_string(self.scales.weight_0());
        let w1 = opt_f64_to_string(self.scales.weight_1());
        let wt = opt_f64_to_string(self.scales.weight_total());

        println!(
            "{} :: weight: ({} + {} = {}) | (task: {} | {}) : (plates: {}) : (ss_id: {})", 
            chrono_now, w0, w1, wt, trigger, peak, self.plate_count, self.service.state().index()
        );
    }
}

fn main() {
    telemetry::init();

    // config
    let config = service::Config {
        config_path: "/home/qitech/config.json".to_string(),
        reconnect_attempts_max: 10,
        timeout_reconnect: Duration::from_millis(2500),
        timeout_heartbeat: Duration::from_millis(60_000),
        timeout_sending:   Duration::from_millis(100),
    };

    let mut app = App {
        plate_count:   0,
        last_print_ts: Instant::now(),
        scales:        Scales::new(),
        service:       Service::new(config),
        task:          None,
    };

    app.service.set_enabled(true);

    let update_freq = 1.0 / 12.0;

    // start
    loop {
        let now = Instant::now();
        app.update(now);

        std::thread::sleep(Duration::from_secs_f64(update_freq));
    }
}

fn opt_f64_to_string(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1}", x),
        None => "_._".to_string(), // or "NaN"
    }
}