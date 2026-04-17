use anyhow::anyhow;
use chrono::{Datelike, Timelike};

use beas_bsl::{
    Client,
    api::{ Date, FilterBuilder, QueryOptions, Time, TimeReceipt },
};

use crate::{service::state::StateTwo, telemetry::{self, LogLevel}};

use super::super::types::Entry;
use super::State;

#[derive(Debug, Clone)]
pub struct StateOne {
    pub entry:      Entry,
    pub start_date: Date,
    pub from_time:  Time,
}

pub fn get_next_state(client: &Client, state: StateOne) -> anyhow::Result<State> {
    let maybe_submission = get_worker_submission(
        client, 
        state.entry.doc_entry, 
        state.entry.line_number
    )?;

    let Some((personnel_id, quantity_scrap)) = maybe_submission else {
        // no entry found
        return Ok(State::One(state));
    };

    let now = chrono::Local::now();
    let end_date = Date { year: now.year(), month: now.month(), day: now.day() };
    let to_time  = Time { hour: now.hour(), minute: now.minute() };

    let state = StateTwo { 
        state_one: state, 
        personnel_id, 
        quantity_scrap,
        end_date,
        to_time
    };

    telemetry::log(LogLevel::Info, format!("State Transition 1 -> 2: {:?}", &state));

    Ok(State::Two(state))
}

fn get_worker_submission(
    client: &Client,
    doc_entry: i32,
    line_number: i32,
) -> anyhow::Result<Option<(String, f64)>> {
    for time_receipt in get_time_receipts(client, doc_entry, line_number)? {
        let Some(quantity_scrap) = time_receipt.quantity_scrap else {
            continue;
        };

        if quantity_scrap == 0.0 {
            continue;
        }

        let Some(personnel_id) = time_receipt.personnel_id else {
            continue;
        };

        return Ok(Some((personnel_id, quantity_scrap)));
    }

    Ok(None)
}

fn get_time_receipts(
    client: &Client,
    doc_entry: i32,
    line_number: i32,
) -> anyhow::Result<Vec<TimeReceipt>> {
    let filter = FilterBuilder::new()
        .equals("DocEntry", doc_entry)
        .and()
        .equals("LineNumber", line_number)
        .and()
        .equals("LineNumber2", 10)
        .build();

    let options = QueryOptions::new().filter(filter);

    let result = client
        .request_single()
        .production()
        .time_receipt()
        .get(options);

    result.map_err(|e| anyhow!("get_time_receipts::Error -> {}", e))
}
