use std::{f64::NAN, time::{Duration, Instant}};

mod telemetry;

mod xtrem;
mod scales;

mod service;
use service::Service;

mod plate_detect_task;
use plate_detect_task::PlateDetectTask;

use crate::{scales::Scales, service::{State, StateOne}, telemetry::{LogLevel, OrderRecord, PlateRecord, Record, WeightEntry, WeightRecord}};

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
                order_id:     0,
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

        let task = self.task.as_mut().unwrap();

        if let Some((peak, drop)) = task.check(weight_total) {
            let exit = weight_total;
            let in_bounds = 
                entry.weight_bounds.min <= weight_total 
                && weight_total <= entry.weight_bounds.max;

            telemetry::record_weight(PlateRecord { peak, drop, exit, in_bounds });

            plate_count += 1;
        }
    }

    fn handle_state_one(&mut self, state: StateOne) {
        let entry = state.entry;

        // init task 
        if self.task.is_none() {
            let min     = entry.weight_bounds.min;
            let trigger = min * 0.8;
            self.task = Some(PlateDetectTask::new(trigger));
            telemetry.send(Entry::Order(Some(entry.doc_entry))).expect("Failed to Send");
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

    let app = App {
        plate_count:   0,
        last_print_ts: Instant::now(),
        scales:        Scales::new(),
        service:       Service::new(config),
        task:          None,
    };

    app.service.set_enabled(true);

    // start
    loop {
        std::thread::sleep(Duration::from_secs_f64(1.0 / 12.0));

        let now = Instant::now();

        scales.update();

        if now.duration_since(last_print_ts) > Duration::from_millis(1000) {
            let (trigger, peak) = match &task {
                Some(value) => (value.trigger, value.peak.unwrap_or(0.0)),
                None => (0.0, 0.0),
            };

            let chrono_now = chrono::Local::now();

            let w0 = opt_f64_to_string(scales.weight_0());
            let w1 = opt_f64_to_string(scales.weight_1());
            let wt = opt_f64_to_string(scales.weight_total());

            println!(
                "{} :: weight: ({} + {} = {}) | (task: {} | {}) : (plates: {}) : (ss_id: {})", 
                chrono_now, w0, w1, wt, trigger, peak, plate_count, service.state().index()
            );

            last_print_ts = now;
        }

        let state_modified = match service.update(now, plate_count) {
            Ok(v) => v,
            Err(e) => {
                let msg = format!("Error while updating service: {}", e);
                println!("{}", msg);
                telemetry::log(LogLevel::Error, msg);
                continue;
            },
        };

        let entry = match service.state() {
            State::One(state) => &state.entry,
            State::Zero | State::Two(_) => {
                task = None;
                if service.state().index() == 0 {
                    plate_count = 0;
                }
                
                telemetry::record_weight(WeightRecord {
                    weight_0:     scales.weight_0().unwrap_or(-123.546789),
                    weight_1:     scales.weight_1().unwrap_or(-123.546789),
                    weight_total: scales.weight_total().unwrap_or(-123.546789),
                    order_id: 0,
                });

                continue;
            }
        };

        assert!(task.is_none());
        // entered state one
        let min     = entry.weight_bounds.min;
        let trigger = min * 0.8;
        task = Some(PlateDetectTask::new(trigger));

        telemetry::record_order(Some(OrderRecord {
            id: entry.doc_entry,
            weight_min: entry.weight_bounds.min,
            weight_max: entry.weight_bounds.max,
            weight_desired: entry.weight_bounds.desired,
            weight_trigger: entry.weight_bounds.min * 0.8,
        }));

        // init task 
        if task.is_none() {
            let min     = entry.weight_bounds.min;
            let trigger = min * 0.8;
            task = Some(PlateDetectTask::new(trigger));

            telemetry.send(Entry::Order(Some(entry.doc_entry))).expect("Failed to Send");
        }

        let task = task.as_mut().unwrap();

        let weight_total = scales.weight_total().expect("Scales disconnected!");

        if let Some((peak, drop)) = task.check(weight_total) {
            let exit = weight_total;
            let in_bounds = 
                entry.weight_bounds.min <= weight_total 
                && weight_total <= entry.weight_bounds.max;

            telemetry::record_weight(PlateRecord { peak, drop, exit, in_bounds });

            plate_count += 1;
        }
    }
}

fn handle_state_one(state: StateOne) {
    if 
}

fn opt_f64_to_string(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.1}", x),
        None => "_._".to_string(), // or "NaN"
    }
}