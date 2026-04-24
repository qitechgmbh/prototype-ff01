use std::{
    io::{self, Cursor, Read, Seek, Write},
    time::Duration,
};

use telemetry_system::{
    ArchiveManagerConfig, Config, RecorderConfig, TelemetrySystem,
    archive::{self, ArchiveTier, FragmentBody},
};

#[derive(Debug)]
pub enum ExampleRecord {
    Any(u32),
}

#[derive(Debug, Clone)]
pub struct ExampleFragmentBody {
    range: (u64, u64),
    bytes: Vec<u8>,
}

impl FragmentBody for ExampleFragmentBody {
    type Record = ExampleRecord;

    fn new(ts: u64) -> Self {
        Self {
            range: (ts, ts + 10),
            bytes: vec![0, 1, 2, 3],
        }
    }

    fn apply(&mut self, record: Self::Record) {
        match record {
            ExampleRecord::Any(x) => {
                self.bytes[0] = x as u8;
            }
        }
    }

    fn range(&self) -> (u64, u64) {
        self.range
    }

    // Encode the body (bytes) of the fragment to the writer
    fn encode_body<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u64_le(self.bytes.len() as u64)?; // Use `?` for error propagation
        writer.write_all(&self.bytes)?; // Same here for error propagation
        Ok(())
    }

    // Decode a fragment from the reader
    fn decode<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf8 = [0u8; 8]; // Buffer to hold the u64 length

        reader.read_exact(&mut buf8)?;
        let size = u64::from_le_bytes(buf8); // Read the length as a u64

        let mut bytes = vec![0u8; size as usize]; // Prepare a vector of the correct size
        reader.read_exact(&mut bytes)?;

        Ok(Self { range, bytes })
    }

    fn merge(&self, other: &Self) -> Self {
        // Check that the ranges are contiguous
        if self.range.1 != other.range.0 {
            panic!("Ranges are not contiguous! Cannot merge.");
        }

        let mut merged = self.clone(); // Clone the current fragment to merge into

        // Concatenate the bytes of the current fragment and the other one
        merged.bytes.extend_from_slice(&other.bytes);

        // Update the range: the new range is from `self.range.0` to `other.range.1`
        merged.range = (self.range.0, other.range.1);

        merged
    }
    
    fn append(&mut self, record: Self::Record) {
        todo!()
    }
    
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        todo!()
    }
    
    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        todo!()
    }
    
    fn stitch<W: Write>(pieces: [&Self], writer: &mut W) -> io::Result<()> {
        todo!()
    }
}

type Fragment = archive::Fragment<ExampleFragmentBody>;

pub fn main() {
    let mut buffer: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    let mut writer = archive::Writer::new(&mut cursor).unwrap();

    let mut fragment = Fragment::new();

    ExampleFragmentBody {
        range: (0, 0),
        bytes: vec![1, 2, 3, 4],
    };

    writer.write_fragment(&fragment).unwrap();

    fragment.range = (1, 2);
    writer.write_fragment(&fragment).unwrap();

    fragment.range = (2, 3);
    writer.write_fragment(&fragment).unwrap();

    fragment.range = (3, 4);

    let mut f = fragment.clone();
    f.range = (4, 5);

    let m = fragment.merge(&f);
    writer.write_fragment(&m).unwrap();

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    let mut reader = archive::Reader::new(&mut cursor).expect("idk");

    loop {
        let Some(frag) = reader.next_fragment::<ExampleFragmentBody>().unwrap() else {
            break;
        };

        println!("frag: {:?}", frag);
    }

    if true {
        return;
    }

    let config = Config {
        recorder: RecorderConfig {
            cycle_time: Duration::from_secs(2),
        },
        archive: ArchiveManagerConfig {
            archive_dir: "/home/entity/sandbox/ff01/machine/telemetry".into(),
            tiers: vec![
                ArchiveTier {
                    triggger: 5,
                    capacity: 3,
                },
                ArchiveTier {
                    triggger: 5,
                    capacity: 3,
                },
                ArchiveTier {
                    triggger: 5,
                    capacity: 3,
                },
            ],
        },
    };

    let telemetry = TelemetrySystem::<ExampleFragmentBody>::start(config).unwrap();

    loop {
        // seed from current time
        let mut seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();

        let number = next_rand(&mut seed) % 6;

        // telemetry
        telemetry
            .record_tx
            .send(ExampleRecord::Any(number))
            .unwrap();

        // super::log(LogLevel::Info, "Hello World".to_string());
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
