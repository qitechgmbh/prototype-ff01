use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    loop {
        let res = client
            .get("http://localhost:9000/api/telemetry/live")
            .send()?;

        let body = res.text()?;

        println!("response: {}", body);

        thread::sleep(Duration::from_secs(1));
    }
}