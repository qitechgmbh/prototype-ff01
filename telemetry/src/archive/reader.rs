use std::{fmt::Debug, io::{self, Cursor, Read, Seek, SeekFrom}};

use crate::{archive::{FragmentBody, MAGIC, VERSION, compute_checksum}, types::Buffer};

use super::{Fragment, ArchiveHeader, FragmentHeader};

#[derive(Debug)]
pub struct Reader<'a, R: Read + Seek> {
    stream: &'a mut R,
    header: ArchiveHeader,
    index:  u32,
}

impl<'a, R: Read + Seek + Debug> Reader<'a, R> {
    pub fn new(stream: &'a mut R) -> io::Result<Self> {
        // --- root header ---
        let header_root = ArchiveHeader::read(stream)?;

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

        let mut prev: Option<FragmentHeader> = None;

        // --- validate fragment chain ---
        for _ in 0..header_root.fragment_count {
            let header_frag = FragmentHeader::read(stream)?;

            // ordering check
            if let Some(prev) = &prev {
                if prev.metadata.to > header_frag.metadata.from || 
                    prev.metadata.to >= header_frag.metadata.to {

                    println!("WHAT: {:?}, {:?}", prev.metadata, header_frag.metadata);

                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "fragment overlap detected",
                    ));
                }
            }

            // basic sanity checks
            if header_frag.metadata.from > header_frag.metadata.to {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid time range in fragment",
                ));
            }

            // skip body
            stream.seek(SeekFrom::Current(header_frag.size as i64))?;

            prev = Some(header_frag);
        }

        // --- rewind to first fragment header ---
        stream.seek(SeekFrom::Start(
            ArchiveHeader::BYTE_SIZE as u64, // root header + fragment count
        ))?;

        Ok(Self {
            stream,
            header: header_root,
            index: 0,
        })
    }

    pub fn header(&self) -> &ArchiveHeader {
        &self.header
    }

    pub fn next_fragment<const N: usize, F: FragmentBody<N>>(
        &mut self
    ) -> io::Result<Option<Fragment<N, F>>> {
        if self.index >= self.header.fragment_count {
            return Ok(None);
        }

        // --- read header ---
        let header = FragmentHeader::read(&mut self.stream)?;

        if header.size == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid fragment size",
            ));
        }

        let mut buffers: Vec<Buffer<'static>> = Vec::new();
        for buf_type in F::LAYOUT {
            
        }

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

        Ok(Some(Fragment::import(header.metadata, body)))
    }
}