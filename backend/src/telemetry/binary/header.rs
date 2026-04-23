use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveHeader {
    // identity / validation
    pub magic: u64,
    pub version: u16,

    // layout
    pub fragment_count: u32,
    pub data_size: u64,
}

impl ArchiveHeader {
    pub const BYTE_SIZE: usize = 8 + 2 + 4 + 8; // 22 bytes

    pub fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.magic.to_le_bytes())?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.fragment_count.to_le_bytes())?;
        writer.write_all(&self.data_size.to_le_bytes())?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut magic_buf = [0u8; 8];
        let mut version_buf = [0u8; 2];
        let mut frag_buf = [0u8; 4];
        let mut size_buf = [0u8; 8];

        reader.read_exact(&mut magic_buf)?;
        reader.read_exact(&mut version_buf)?;
        reader.read_exact(&mut frag_buf)?;
        reader.read_exact(&mut size_buf)?;

        Ok(Self {
            magic: u64::from_le_bytes(magic_buf),
            version: u16::from_le_bytes(version_buf),
            fragment_count: u32::from_le_bytes(frag_buf),
            data_size: u64::from_le_bytes(size_buf),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentHeader {
    // integrity
    pub checksum: u64,

    // range
    pub from: u64,
    pub to:   u64,

    // layout
    pub size: u64,
}

impl FragmentHeader {
    pub const BYTE_SIZE: usize = 8 + 8 + 8 + 8; // 32 bytes

    pub fn encode<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.checksum.to_le_bytes())?;
        writer.write_all(&self.from.to_le_bytes())?;
        writer.write_all(&self.to.to_le_bytes())?;
        writer.write_all(&self.size.to_le_bytes())?;
        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut checksum_buf = [0u8; 8];
        let mut from_buf     = [0u8; 8];
        let mut to_buf       = [0u8; 8];
        let mut size_buf     = [0u8; 8];

        reader.read_exact(&mut checksum_buf)?;
        reader.read_exact(&mut from_buf)?;
        reader.read_exact(&mut to_buf)?;
        reader.read_exact(&mut size_buf)?;

        Ok(Self {
            checksum: u64::from_le_bytes(checksum_buf),
            from: u64::from_le_bytes(from_buf),
            to: u64::from_le_bytes(to_buf),
            size: u64::from_le_bytes(size_buf),
        })
    }
}