use std::{borrow::Cow, fmt::Debug};

use crate::format::Schema;

#[derive(Debug, Clone)]
pub struct Header {
    pub checksum: u64,
    pub metadata: Metadata,
    pub size:     u64,
}

#[derive(Debug, Clone)]
pub struct Fragment<Body: FragmentBody> {
    metadata: Metadata,
    body:     Box<Body>,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub from: u64,
    pub to:   u64,
}

pub trait FragmentBody where Self: Debug {
    const SCHEMA: Schema;
    fn tables(&self) -> &[TableData];
}

#[derive(Debug, Clone)]
pub struct TableData<'a> {
    columns: &'a [ColumnData<'a>]
}

#[derive(Debug, Clone)]
pub enum ColumnData<'a> {
    Unsigned8(Cow<'a, [u8]>),
    Unsigned16(Cow<'a, [u16]>),
    Unsigned32(Cow<'a, [u32]>),
    Unsigned64(Cow<'a, [u64]>),

    Signed8(Cow<'a, [i8]>),
    Signed16(Cow<'a, [i16]>),
    Signed32(Cow<'a, [i32]>),
    Signed64(Cow<'a, [i64]>),

    Float32(Cow<'a, [f32]>),
    Float64(Cow<'a, [f64]>),

    String(Cow<'a, [String]>),
}

/*
use std::{fmt::Debug, io::{self, Read, Write}};

use zip::unstable::{LittleEndianReadExt, LittleEndianWriteExt};

use crate::types::{Buffer, BufferType};

// archive_manager receives layout -> then send

#[derive(Debug, Clone)]
pub struct Fragment<const N: usize, B: Body<N>> {
    metadata: Metadata,
    body: B,
}

impl<const N: usize, B: Body<N>> Fragment<N, B> {
    pub fn new(now: u64) -> Self {
        Self { 
            metadata: Metadata {
                from: now,
                to:   now,
            },
            body: Default::default(), 
        }
    }

    pub(crate) fn import(metadata: Metadata, body: B) -> Self {
        Self { metadata, body }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn body(&self) -> &B {
        &self.body
    }

    pub fn append(&mut self, now: u64, record: B::Record) -> Result<(), ()> {
        if now < self.metadata.to {
            return Err(());
        }
        self.metadata.to = now;
        self.body.append_record(now, record);
        Ok(())
    }

    /*
    pub fn try_merge(&self, other: &Self) -> Option<Fragment<B>> {
        if self.metadata.to != other.metadata.from {
            return None;
        }

        let merged_meta = Metadata {
            from: self.metadata.from,
            to:   other.metadata.to,
        };

        // let merged_body = self.body.merge(&other.body);
        // todo!()

        // Some(Fragment {
        //     metadata: merged_meta,
        //     body:     merged_body,
        // })
    }
    */
}

#[derive(Debug, Clone)]
pub struct Header {
    pub checksum: u64,
    pub metadata: Metadata,
    pub size:     u64,
}

impl Header {
    // Write the header to a writer
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_u64_le(self.checksum)?;
        writer.write_u64_le(self.metadata.from)?;
        writer.write_u64_le(self.metadata.to)?;
        writer.write_u64_le(self.size)?;

        Ok(())
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let checksum = reader.read_u64_le()?;
        let from     = reader.read_u64_le()?;
        let to       = reader.read_u64_le()?;
        let size     = reader.read_u64_le()?;

        Ok(Header {
            checksum,
            metadata: Metadata { from, to },
            size,
        })
    }

    pub(crate) fn write_dummy<W: Write>(writer: &mut W) -> io::Result<()> {
        let dummy_checksum = 0xDEADBEEF;
        let dummy_metadata = Metadata { from: 0, to: 0 };
        let dummy_size     = 0;

        writer.write_u64_le(dummy_checksum)?;
        writer.write_u64_le(dummy_metadata.from)?;
        writer.write_u64_le(dummy_metadata.to)?;
        writer.write_u64_le(dummy_size)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub from: u64,
    pub to:   u64,
}

pub trait Body<const N: usize>: Sized + Send + Sync + Debug + Default + Clone + 'static {
    type Record;
    const LAYOUT: [BufferType; N];
    fn append_record(&mut self, ts: u64, record: Self::Record);
    fn buffers(&self) -> [Buffer<'_>; N];
    fn from_buffers(buffers: [Buffer<'_>; N]) -> Self;
}


// Table { timestamp: &[BufferType] } 
// read(layout: [BufferType])
// write(buffers: &[Buffer<'_>]);
// 
*/