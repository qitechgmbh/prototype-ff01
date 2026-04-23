use std::{fs::{File, OpenOptions}, io::{self, Read, Seek, SeekFrom}, path::PathBuf};

use crate::telemetry::binary::{ArchiveHeader, Fragment, FragmentHeader, checksum};

pub struct ArchiveWriter {
    file: File,
    path: PathBuf,
    header: ArchiveHeader,
}

impl ArchiveWriter {
    pub fn create(path: PathBuf, magic: u64, version: u16) -> io::Result<Self> {
        let path = path.with_extension("tmp");

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;

        let header = ArchiveHeader {
            magic,
            version,
            fragment_count: 0,
            data_size: 0,
        };

        header.encode(&mut file)?;

        Ok(Self {
            file,
            path,
            header,
        })
    }

    pub fn write_fragment<T: Fragment>(
        &mut self,
        fragment: &T,
    ) -> io::Result<()> {
        let (from, to) = fragment.range();

        // 1. remember header position
        let header_pos = self.file.stream_position()?;

        // 2. write placeholder header
        FragmentHeader {
            checksum: 0,
            from,
            to,
            size: 0,
        }.encode(&mut self.file)?;

        // 3. remember body start
        let body_start = self.file.stream_position()?;

        // 4. write body
        fragment.encode_body(&mut self.file)?;

        // 5. compute size
        let body_end = self.file.stream_position()?;
        let size = body_end - body_start;

        // 6. compute checksum (requires reread OR streaming version)
        self.file.seek(io::SeekFrom::Start(body_start))?;

        let mut buf = vec![0u8; size as usize];
        self.file.read_exact(&mut buf)?;
        let checksum = checksum(&buf);

        // 7. go back and patch header
        self.file.seek(io::SeekFrom::Start(header_pos))?;

        FragmentHeader {
            checksum,
            from,
            to,
            size,
        }
        .encode(&mut self.file)?;

        // 8. return to end
        self.file.seek(io::SeekFrom::Start(body_end))?;

        Ok(())
    }

    pub fn finalize(self) -> io::Result<PathBuf> {
        let path = self.path.clone();
        match self.finalize_inner() {
            Ok(v) => Ok(v),
            Err(e) => {
                let _ = std::fs::remove_file(&path);
                Err(e)
            }
        }
    }

    fn finalize_inner(mut self) -> io::Result<PathBuf> {
        // --- rewrite header ---
        self.file.seek(SeekFrom::Start(0))?;
        self.header.encode(&mut self.file)?;

        // IMPORTANT: ensure data is on disk
        self.file.sync_all()?;

        // --- atomic rename ---
        let final_path = self.path.with_extension("qta");

        std::fs::rename(&self.path, &final_path)?;

        Ok(final_path)
    }
}