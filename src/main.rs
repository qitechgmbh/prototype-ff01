use std::{f64::NAN, time::{Duration, Instant}};

mod telemetry;
use telemetry::LogLevel;
use telemetry::WeightBoundsRecord;
use telemetry::PlateRecord ;
use telemetry::WeightRecord;

mod xtrem;

mod scales;
use scales::Scales;

mod service;
use service::Service;
use service::State;

mod plate_detect_task;
use plate_detect_task::PlateDetectTask;

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

        let wt = {
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
            let wt = w0 + w1;

            telemetry::record_weight(WeightRecord {
                weight_0:     w0,
                weight_1:     w1,
                weight_total: wt,
            });

            if !complete {
                return;
            }

            wt
        };

        let prev_state_idx = self.service.state().index();

        if let Err(e) = self.service.update(now, self.plate_count) {
            let msg = format!("Error while updating service: {}", e);
            println!("{}", msg);
            telemetry::log(LogLevel::Error, msg);
            return;
        }

        let state_idx = self.service.state().index();
        let state_modified = prev_state_idx != state_idx;

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
                telemetry::record_bounds(WeightBoundsRecord {
                    order_id: entry.doc_entry,
                    min:      entry.weight_bounds.min,
                    max:      entry.weight_bounds.max,
                    desired:  entry.weight_bounds.desired,
                    trigger:  trigger,
                });
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

        let w0 = self.scales.weight_0().unwrap_or(NAN);
        let w1 = self.scales.weight_1().unwrap_or(NAN);
        let wt = self.scales.weight_total().unwrap_or(NAN);

        let entry_id: i32 = match self.service.state() {
            State::Zero => 0,
            State::One(state) => state.entry.doc_entry,
            State::Two(state) => state.state_one.entry.doc_entry,
        };

        println!(
            "{} :: scales: [w0: {:.1}, w1: {:.1}, wt: {:.1}] | task: [trigger: {:.1},  peak: {:.1}, count: {}] | service: [state_id: {}, order_id: {}]", 
            chrono_now, w0, w1, wt, trigger, peak, self.plate_count, self.service.state().index(), entry_id
        );
    }
}

fn main() {
    telemetry::init();

    // config
    let config = service::Config {
        config_path: "/home/qitech/config.json".to_string(),
        reconnect_attempts_max: 10,
        reconnect_delay:  Duration::from_secs(2),
        timeout_duration: Duration::from_secs(60),
        send_delay:       Duration::from_millis(250),
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