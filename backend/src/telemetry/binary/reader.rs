use std::{fs::File, io::{self, Cursor, Read, Seek, SeekFrom}, path::Path};

use crate::telemetry::binary::{ArchiveHeader, Fragment, FragmentHeader, MAGIC, VERSION, checksum};

enum FragmentReadResult<T> {
    Ok(T),
    Corrupt(FragmentHeader),
}

#[derive(Debug)]
pub struct ArchiveReader {
    file:   File,
    header: ArchiveHeader,
    index:  u32,
}

impl ArchiveReader {
    pub fn open(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;

        // --- root header ---
        let header_root = ArchiveHeader::read(&mut file)?;

        // validate identity early (cheap fail)
        if header_root.magic != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid magic",
            ));
        }

        if header_root.version != VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported version",
            ));
        }

        // --- fragment count ---
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf)?;
        let fragment_count = u32::from_le_bytes(buf);

        let mut prev: Option<FragmentHeader> = None;

        // --- validate fragment chain ---
        for _ in 0..fragment_count {
            let header_frag = FragmentHeader::read(&mut file)?;

            // ordering check
            if let Some(prev) = &prev {
                if prev.to > header_frag.from {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "fragment overlap detected",
                    ));
                }
            }

            // basic sanity checks
            if header_frag.from > header_frag.to {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid time range in fragment",
                ));
            }

            // skip body
            file.seek(SeekFrom::Current(header_frag.size as i64))?;

            prev = Some(header_frag);
        }

        // --- rewind to first fragment header ---
        file.seek(SeekFrom::Start(
            ArchiveHeader::BYTE_SIZE as u64 + 4, // root header + fragment count
        ))?;

        Ok(Self {
            file,
            header: header_root,
            index: 0,
        })
    }

    pub fn next<T: Fragment>(&mut self) -> io::Result<Option<T>> {
        if self.index >= self.header.fragment_count {
            return Ok(None);
        }

        // --- read header ---
        let header = FragmentHeader::read(&mut self.file)?;

        if header.size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid fragment size",
            ));
        }

        // --- read body ---
        let mut body = vec![0u8; header.size as usize];
        self.file.read_exact(&mut body)?;

        // --- checksum validation ---
        let computed = checksum(&body);

        if computed != header.checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "checksum mismatch (corrupted fragment)",
            ));
        }

        self.index += 1;

        let mut reader = Cursor::new(body);
        let instance = T::decode(&mut reader, (header.from, header.to))?;
        Ok(Some(instance))
    }
}