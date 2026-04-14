// use anyhow::anyhow;
use beas_bsl::{Client, api::{Date, Time, time_receipt}};
use chrono::{Datelike, Local, Timelike};

use crate::{service::State, telemetry::{self, LogLevel}};

use super::StateOne;

#[derive(Debug, Clone)]
pub struct StateTwo {
    pub state_one: StateOne,
    pub personnel_id: String,
    pub quantity_scrap: f64,
}

pub fn get_next_state(
    client: &Client, 
    state: StateTwo, 
    plate_count: u32
) -> anyhow::Result<State> {
    _ = client;

    let s1 = &state.state_one;
    let s2 = &state;

    let doc_entry = s1.entry.doc_entry;

    let start_date = s1.start_date;
    let from_time  = s1.from_time;

    let personnel_id   = s2.personnel_id.clone();
    let quantity_scrap = s2.quantity_scrap;

    let chrono_now = Local::now();
    let end_date   = Date { year: chrono_now.year(), month: chrono_now.month(), day: chrono_now.day() };
    let to_time    = Time { hour: chrono_now.hour(), minute: chrono_now.minute() };

    let quantity_good = plate_count as f64 - quantity_scrap;
    let quantity_good = quantity_good.max(0.0);

    let duration = from_time.compute_duration(to_time);

    let request = time_receipt::post::Request {
        doc_entry:          doc_entry,
        line_number:        10,
        line_number2:       10,
        line_number3:       Some(0),
        time_type:          Some("A".to_string()),
        resource_id:        Some("FF01".to_string()),
        quantity_good:      Some(quantity_good),
        personnel_id:       personnel_id,
        quantity_scrap:     Some(0.0),
        start_date:         Some(start_date),
        end_date:           Some(end_date),
        from_time:          Some(from_time),
        to_time:            Some(to_time),
        close_entry:        Some(true),
        manual_booking:     Some(false),
        duration:           Some(duration.as_secs_f32()),
        calculate_duration: Some(false),
        remarks:            Some("QiTech-Control".to_string()),
        ..Default::default()
    };

    telemetry::log(LogLevel::Info, format!("Posting TimeReceipt: {:?}", &request));
    telemetry::log(LogLevel::Info, format!("State Transition 2 -> 0"));

    Ok(State::Zero)
}

/* 
fn post_time_receipt(
    client: &Client,
    request: time_receipt::post::Request,
) -> anyhow::Result<time_receipt::post::Response> {
    panic!("OPERATION NOT ALLOWED");

    let result = client
        .request_single()
        .production()
        .time_receipt()
        .post(request);

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(anyhow!("get_workorder_routing::Error -> {}", e)),
    }
}
*/