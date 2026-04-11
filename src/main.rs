use std::{time::{Duration, Instant}};

mod telemetry;

mod xtrem;
mod scales;

mod service;
use service::Service;

mod plate_detect_task;
use plate_detect_task::PlateDetectTask;

use crate::{scales::Scales, telemetry::{Message, PlateMessage, WeightMessage}};

fn main() {
    let handle = telemetry::start();
    telemetry::HANDLE.set(handle.clone()).ok();

    let telemetry = handle.clone();

    // config
    let service_config = service::Config {
        config_path: "/home/qitech/config.json".to_string(),
        reconnect_attempts_max: 10,
        timeout_reconnect: Duration::from_millis(2500),
        timeout_heartbeat: Duration::from_millis(60_000),
        timeout_sending:   Duration::from_millis(100),
    };

    // state
    let mut plate_count: u32 = 0;
    let mut last_print_ts = Instant::now();

    // components
    let mut scales  = Scales::new();
    let mut service = Service::new(service_config);
    let mut task: Option<PlateDetectTask> = None;

    // init
    service.set_enabled(true);

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

        if let Err(e) = service.update(now, plate_count) {

            telemetry.send(Message::Event(telemetry::EventMessage { event_type: telemetry::EventType::Error, message: format!("Error in response: {}", e) })).expect("Oh no no send can me");
            println!("Error while updating service: {}", e);
        }
        
        let entry = match service.state() {
            service::State::Zero => {
                task        = None;
                plate_count = 0;

                telemetry.send(Message::Order(None)).expect("Oh no no no");
                telemetry.send(Message::Weight(WeightMessage { 
                    weight_0:       scales.weight_0().unwrap_or(-99.0), 
                    weight_1:       scales.weight_0().unwrap_or(-99.0), 
                    weight_total:   scales.weight_0().unwrap_or(-99.0), 
                    weight_min:     0.0, 
                    weight_max:     0.0,
                    weight_desired: 0.0,
                })).expect("Failed to Send");

                continue;
            },
            service::State::One(state) => &state.entry,
            service::State::Two(_) => {
                task = None;

                telemetry.send(Message::Order(None));
                telemetry.send(Message::Weight(WeightMessage { 
                    weight_0:     scales.weight_0().unwrap_or(-99.0), 
                    weight_1:     scales.weight_0().unwrap_or(-99.0), 
                    weight_total: scales.weight_0().unwrap_or(-99.0), 
                    weight_min:     0.0, 
                    weight_max:     0.0,
                    weight_desired: 0.0,
                })).expect("Failed to Send");

                continue;
            },
        };

        telemetry.send(Message::Weight(WeightMessage { 
            weight_0:       scales.weight_0().unwrap_or(-99.0), 
            weight_1:       scales.weight_0().unwrap_or(-99.0), 
            weight_total:   scales.weight_0().unwrap_or(-99.0), 
            weight_min:     entry.weight_bounds.min, 
            weight_max:     entry.weight_bounds.max,
            weight_desired: entry.weight_bounds.desired,
        })).expect("Failed to Send");

        // init task 
        if task.is_none() {
            let min     = entry.weight_bounds.min;
            let trigger = min * 0.8;
            task = Some(PlateDetectTask::new(trigger));

            telemetry.send(Message::Order(Some(entry.doc_entry))).expect("Failed to Send");
        }

        let task = task.as_mut().unwrap();

        let weight_total = scales.weight_total().expect("Scales disconnected!");

        if let Some((peak, drop)) = task.check(weight_total) {
            let in_bounds = 
                entry.weight_bounds.min <= weight_total 
                && weight_total <= entry.weight_bounds.max;

            telemetry.send(Message::Plate(PlateMessage {
                triggger: task.trigger,
                peak,
                drop,
                exit:       weight_total,
                in_bounds:  if in_bounds { 1 } else { 0 },
            })).expect("Failed to Send");

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