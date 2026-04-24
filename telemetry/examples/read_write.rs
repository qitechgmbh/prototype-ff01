use std::io::{self, Cursor, Read, Seek, Write};

use telemetry_system::archive::{self, FragmentBody};

#[derive(Debug)]
pub enum Record {
    Any(u32)
}

#[derive(Debug, Clone, Default)]
pub struct ExampleFragmentBody {
    number: u32,
}

impl FragmentBody for ExampleFragmentBody {
    type Record = Record;

    fn append(&mut self, record: Self::Record) {
        match record {
            Record::Any(x) => {
                self.number += x;
            }
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.number.to_le_bytes())?;
        Ok(())
    }

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf8 = [0u8; 4];

        reader.read_exact(&mut buf8)?;
        let number = u32::from_le_bytes(buf8);

        Ok(Self { number })
    }
    
    fn merge(&self, other: &Self) -> Self {
        todo!()
    }

    // fn stitch<W: Write>(pieces: [&Self], writer: &mut W) -> io::Result<()> {
    //     todo!()
    // }
}

type Fragment = archive::Fragment<ExampleFragmentBody>;

pub fn main() -> io::Result<()> {
    let mut fragment = Fragment::new(0);
    fragment.append(10, Record::Any(10)).unwrap();
    fragment.append(20, Record::Any(20)).unwrap();
    fragment.append(30, Record::Any(30)).unwrap();

    let mut buffer: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    let mut writer = archive::Writer::new(&mut cursor).unwrap();
    writer.write_fragment(&fragment).unwrap();

    let mut fragment = Fragment::new(100);
    fragment.append(110, Record::Any(40)).unwrap();
    writer.write_fragment(&fragment).unwrap();

    cursor.seek(io::SeekFrom::Start(0)).unwrap();
    let mut reader = archive::Reader::new(&mut cursor).expect("idk");

    loop {
        let Some(frag) = reader.next_fragment::<ExampleFragmentBody>().unwrap() else {
            break;
        };

        println!("frag: {:?}", frag);
    }

    Ok(())
}