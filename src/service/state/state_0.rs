use std::os::linux::raw::stat;

use anyhow::anyhow;
use beas_bsl::{
    Client,
    api::{
        Date, FilterBuilder, QCOrderMeasurement, QueryOptions, Time, WorkorderBom, WorkorderRouting,
    },
};
use chrono::{Datelike, Local, Timelike};

use crate::{service::types::TargetRange, telemetry::{self, EventMessage, EventType, Message}};

use super::super::types::Entry;
use super::{State, StateOne};

pub fn get_next_state(client: &Client) -> anyhow::Result<State> {
    let Some(entry) = get_next_entry(client)? else {
        // no entry found
        return Ok(State::Zero);
    };

    let now = Local::now();
    let start_date = Date {
        year: now.year(),
        month: now.month(),
        day: now.day(),
    };
    let from_time = Time {
        hour: now.hour(),
        minute: now.minute(),
    };

    let state = StateOne {
        entry,
        start_date,
        from_time,
    };

    let telemetry = telemetry::HANDLE.wait().clone();
    telemetry.send(Message::Event(EventMessage {
        event_type: EventType::Info,
        message: format!("State Transition 0 -> 1: {:?}", &state),
    })).expect("What the fuck");

    Ok(State::One(state))
}

fn get_next_entry(client: &Client) -> anyhow::Result<Option<Entry>> {
    // Get Workorder Routing
    let wo_routing = match get_workorder_routing(client)? {
        Some(workorder) => workorder,
        None => return Ok(None),
    };

    let doc_entry = wo_routing.doc_entry;
    let line_number = wo_routing.line_number;

    // Get Workorder Bom
    let Some(wo_bom) = get_workorder_bom(client, doc_entry, line_number)? else {
        return Err(anyhow!("No matching WorkorderRoutings for {}", doc_entry));
    };

    let Some(item_code) = wo_bom.item_code else {
        return Err(anyhow!("ItemCode is null"));
    };

    let Some(whs_code) = wo_bom.whs_code else {
        return Err(anyhow!("WhsCode is null"));
    };

    // Get QCOrder Measurement
    let qcorder_measurement = match get_qcorder_measurement(client, doc_entry, line_number)? {
        Some(value) => value,
        None => return Err(anyhow!("No QCOrderMeasurement")),
    };

    let min = unpack_nullable(qcorder_measurement.minimal, "min")?;
    let max = unpack_nullable(qcorder_measurement.maximum, "max")?;
    let desired = unpack_nullable(qcorder_measurement.desired_value, "desired")?;

    let weight_bounds = TargetRange { min, max, desired };

    // return result
    let entry = Entry {
        doc_entry,
        line_number,
        item_code,
        whs_code,
        weight_bounds,
    };

    Ok(Some(entry))
}

fn get_workorder_routing(client: &Client) -> anyhow::Result<Option<WorkorderRouting>> {
    let filter = FilterBuilder::new()
        .equals("CurrentRunning", true)
        .and()
        .equals("ResourceId", "FF01")
        .and()
        .equals("Closed", false)
        .and()
        .equals("LineNumber2", 10)
        .build();

    let options = QueryOptions::new().filter(filter);

    let result = client
        .request_single()
        .production()
        .workorder_routing()
        .get(options);

    match result {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(anyhow!("get_workorder_routing::Error -> {}", e)),
    }
}

fn get_workorder_bom(
    client: &Client,
    doc_entry: i32,
    line_number: i32,
) -> anyhow::Result<Option<WorkorderBom>> {
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
        .workorder_bom()
        .get(options);

    match result {
        Ok(items) => Ok(items.first().cloned()),
        Err(e) => Err(anyhow!("get_workorder_bom::Error -> {}", e)),
    }
}

fn get_qcorder_measurement(
    client: &Client,
    doc_entry: i32,
    line_number: i32,
) -> anyhow::Result<Option<QCOrderMeasurement>> {
    let filter = FilterBuilder::new()
        .equals("WoDocEntry", doc_entry)
        .and()
        .equals("WoLineNumber", line_number)
        .and()
        .equals("LineNumber2", 10)
        .and()
        .equals("QCDescription", "QiTech_Gewicht")
        .build();

    let options = QueryOptions::new().filter(filter);

    let result = client
        .request_single()
        .quality_control()
        .qcorder_measurement()
        .get(options);

    match result {
        Ok(workorders) => Ok(workorders.first().cloned()),
        Err(e) => Err(anyhow!("get_qcorder_measurement::Error -> {}", e)),
    }
}

fn unpack_nullable<T>(value: Option<T>, name: &'static str) -> anyhow::Result<T> {
    match value {
        Some(item) => Ok(item),
        None => return Err(anyhow!("Received null for {}", name)),
    }
}
