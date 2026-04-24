use std::io::{self, Read, Seek, Write};

use crate::archive::{Fragment, FragmentBody, MAGIC, VERSION};

use super::{ArchiveHeader, FragmentHeader, compute_checksum};

pub struct Writer<'a, S: Write + Read + Seek> {
    header: ArchiveHeader,
    stream: &'a mut S,
}

impl<'a, S: Write + Read  + Seek> Writer<'a, S> {
    pub fn new(stream: &'a mut S) -> io::Result<Self> {
        let header = ArchiveHeader {
            magic:          MAGIC,
            version:        VERSION,
            fragment_count: 0,
            data_size:      0,
        };
        
        header.write(stream)?;
        Ok(Self { header, stream })
    }

    pub fn write_fragment<const N: usize, B: FragmentBody<N>>(
        &mut self,
        fragment: &Fragment<N, B>,
    ) -> io::Result<()> {
        // 1. remember header position
        let header_pos = self.stream.stream_position()?;

        // 2. write placeholder header
        FragmentHeader::write_dummy(&mut self.stream)?;

        // 3. remember body start
        let body_start = self.stream.stream_position()?;

        // 4. write body
        for buffer in fragment.body().buffers() {
            buffer.write(&mut self.stream);
        }

        // 5. compute size
        let body_end = self.stream.stream_position()?;
        let size     = body_end - body_start;

        // 6. compute checksum (requires reread OR streaming version)
        self.stream.seek(io::SeekFrom::Start(body_start))?;

        let mut buf = vec![0u8; size as usize];
        self.stream.read_exact(&mut buf)?;
        let checksum = compute_checksum(&buf);

        // 7. go back and patch header
        self.stream.seek(io::SeekFrom::Start(header_pos))?;

        let header = FragmentHeader {
            checksum,
            metadata: fragment.metadata().clone(),
            size,
        };
        header.write(&mut self.stream);

        // update root header
        self.header.fragment_count += 1;
        self.header.data_size      += size;

        self.stream.seek(io::SeekFrom::Start(0))?;
        self.header.write(&mut self.stream)?;

        // return to end
        self.stream.seek(io::SeekFrom::Start(body_end))?;

        Ok(())
    }
}