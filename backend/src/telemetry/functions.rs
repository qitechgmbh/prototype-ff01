use crate::telemetry::{HANDLE, LogLevel, writer::RecordRequest};

pub fn record_weight(w0: f64, w1: f64) {
    let w0 = (w0 * 10.0) as i16;
    let w1 = (w1 * 10.0) as i16;
    send(RecordRequest::Weight { w0, w1 });
}

pub fn record_plate(peak: f64, avg: f64) {
    let peak = (peak * 10.0) as i16;
    let avg  = (avg  * 10.0) as i16;
    send(RecordRequest::Plate { peak, avg });
}

pub fn record_bounds(min: f64, max: f64, desired: f64, trigger: f64) {
    let min     = (min * 10.0) as i16;
    let max     = (max * 10.0) as i16;
    let desired = (desired * 10.0) as i16;
    let trigger = (trigger * 10.0) as i16;
    send(RecordRequest::Bounds { min, max, desired, trigger });
}

pub fn record_state(order_id: u32, state_id: u32) {
    send(RecordRequest::State { order_id, state_id });
}

#[allow(unused)]
pub fn record_order() {
    /*
    let start_time = format!(
        "{}-{}-{}T{}:{}",
        record.start_date.year,
        record.start_date.month,
        record.start_date.day,
        record.from_time.hour,
        record.from_time.minute,
    );

    let end_time = format!(
        "{}-{}-{}T{}:{}",
        record.end_date.year,
        record.end_date.month,
        record.end_date.day,
        record.to_time.hour,
        record.to_time.minute,
    );

    let data = format!(
        "{}, {}, {}, {:.1}, {:.1}, {}, {}, {:.2}\n",
        get_timestamp(),
        record.id,
        record.personel_id,
        record.quantity_scrap,
        record.quantity_good,
        start_time,
        end_time,
        record.duration.as_secs_f64(),
    );

    send(RecordType::Order, data);
    */

    todo!()
}

pub fn log(level: LogLevel, message: String) {
    send(RecordRequest::Log(level, message));
}

fn send(request: RecordRequest) {
    let tx = HANDLE.get().expect("Failed to retrieve handle");
    tx.send(request).expect("worker channel is full");
}