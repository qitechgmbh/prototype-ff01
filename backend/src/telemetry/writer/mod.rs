use std::{thread, time::{Duration, SystemTime}};

use crossbeam::channel::{Receiver, Sender, TryRecvError};

use crate::telemetry::{segment::DataFragment};

mod types;
pub use types::RecordRequest;

#[allow(unused)]
pub use types::OrderRecord;

#[derive(Debug)]
pub struct Config {
    pub cycle_time: Duration,
}

#[derive(Debug)]
pub struct Writer {
    config: Config,
    record_rx: Receiver<RecordRequest>, 
    segement_tx: Sender<Box<DataFragment>>
}

impl Writer {
    pub fn new(
        config: Config, 
        record_rx: Receiver<RecordRequest>, 
        segement_tx: Sender<Box<DataFragment>>
    ) -> Self {
        Self { config, record_rx, segement_tx }
    }

    pub fn run(self) {
        let mut segment_ts = SystemTime::now();
        let mut segment    = Box::new(DataFragment::new(segment_ts));

        loop {
            let now = SystemTime::now();
            let dt  = now.duration_since(segment_ts).unwrap();

            if dt >= self.config.cycle_time { 
                segment_ts = now;
                let old_segment = std::mem::replace(
                    &mut segment,
                    Box::new(DataFragment::new(segment_ts)),
                );

                println!("Fragment complete: {:?}", segment_ts);

                let segment_tx = self.segement_tx.clone();
                thread::spawn(move || {
                    // let thread block until channel is open
                    let res = segment_tx.send(old_segment);
                    println!("RESULT: {:?}", res);
                    res.expect("Failed to submit");
                });
            }

            loop {
                match self.record_rx.try_recv() {
                    Ok(request) => Self::handle_request(request, dt.as_millis() as u32, &mut *segment),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        panic!("MainThread Channel Disconnected");
                    }
                }
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    fn handle_request(request: RecordRequest, dt: u32, batch: &mut DataFragment) {
        use RecordRequest::*;

        match request {
            Weight { w0, w1 } => {
                batch.append_weight(dt, w0, w1).unwrap();
            }

            Plate { peak, avg } => {
                batch.append_plate(dt, peak, avg);
            }

            Bounds { min, max, desired, trigger } => {
                batch.append_bounds(dt, min, max, desired, trigger);
            }

            State { order_id, state_id } => {
                batch.append_state(order_id, state_id);
            }

            Order(record_order_request) => {
                _ = record_order_request;
                // batch.append_order(
                //     record_order_request.order_id,
                //     &record_order_request.personnel_id,
                //     record_order_request.quantity_scrap,
                //     record_order_request.quantity_good,
                //     &record_order_request.time_start,
                //     &record_order_request.time_end,
                //     record_order_request.duration,
                // );
            }

            Log(log_level, message) => {
                // assuming you will store logs later
                // placeholder: ignore or implement push into LogRecords buffer
                let _ = (log_level, message);
            }
        }
    }
}