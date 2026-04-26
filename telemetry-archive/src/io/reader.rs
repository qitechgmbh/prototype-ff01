use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::fmt::Debug;

use crate::{Fragment, MAGIC, VERSION};
use crate::io::{ArchiveHeader, FragmentHeader};

#[derive(Debug)]
pub struct ArchiveReader<'a, R: Read + Seek> {
    stream: &'a mut R,
    header: ArchiveHeader,
    index:  u32,
}

impl<'a, R: Read + Seek> ArchiveReader<'a, R> {
    pub fn new(stream: &'a mut R) -> io::Result<Self> {
        let header_root = ArchiveHeader::read(stream)?;

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

        let mut prev: Option<FragmentHeader> = None;

        for _ in 0..header_root.fragment_count {
            let header_frag = FragmentHeader::read(stream)?;

            if let Some(prev) = &prev {
                if prev.range.0 > header_frag.range.1 || 
                    prev.range.1 >= header_frag.range.1 {

                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "fragment overlap detected",
                    ));
                }
            }

            if header_frag.range.0 > header_frag.range.1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid time range in fragment",
                ));
            }

            stream.seek(SeekFrom::Current(header_frag.size as i64))?;

            prev = Some(header_frag);
        }

        stream.seek(SeekFrom::Start(ArchiveHeader::BYTE_SIZE as u64))?;

        Ok(Self {
            stream,
            header: header_root,
            index: 0,
        })
    }

    pub fn next_fragment<const N: usize, T: Fragment>(
        &mut self
    ) -> io::Result<Option<T>> {
        if self.index >= self.header.fragment_count {
            return Ok(None);
        }

        let header = FragmentHeader::read(&mut self.stream)?;

        if header.size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid fragment size",
            ));
        }

        // allow reading 

        // --- read body ---
        let mut body = vec![0u8; header.size as usize];
        self.stream.read_exact(&mut body)?;

        // --- checksum validation ---
        let computed = compute_checksum(&body);

        if computed != header.checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "checksum mismatch (corrupted fragment)",
            ));
        }

        self.index += 1;

        let mut reader = Cursor::new(body);
        let body = F::from_buffers(buffers);
        // Box::new(F::read(&mut reader)?);

        // Ok(Some(Fragment::import(header.metadata, body)))
        todo!()
    }
}

#[derive(Debug)]
pub struct FragmentReader<'a, R: Read + Seek> {
    stream: &'a mut R,
    header: ArchiveHeader,
    index:  u32,
}