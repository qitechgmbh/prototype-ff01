#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::telemetry::{self, ArchiveManagerConfig, ServerConfig, WriterConfig};

    #[test]
    pub fn my_test() {
        let config = telemetry::Config {
            writer: WriterConfig {
                cycle_time: Duration::from_secs(5),
            },
            server: ServerConfig {
                port: 9000,
            },
            archive: ArchiveManagerConfig {
                archive_dir: "/home/entity/sandbox/ff01/machine/telemetry".into(),
                tiers:      vec![0, 2, 2],
            },
        };

        telemetry::init(config);

        loop {
            // seed from current time
            let mut seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos();

            let w0 = (next_rand(&mut seed) % 6) as f64;
            let w1 = (next_rand(&mut seed) % 6) as f64;

            telemetry::record_weight(w0, w1);

            //super::log(LogLevel::Info, "Hello World".to_string());
            std::thread::sleep(Duration::from_millis(1000 / 12));
        }
    }

    fn next_rand(seed: &mut u32) -> u32 {
        let mut x = *seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *seed = x;
        x
    }
}