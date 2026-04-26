use std::{io::{self, Read, Write}, marker::PhantomData};

use crate::{Column, Fragment, FragmentHeader};

pub trait FragmentReader<'a, T: Fragment> {
    fn header(&self) -> FragmentHeader;
    fn position(&self) -> (u32, u32);
    fn get_next_column(&'a mut self) -> io::Result<Option<Column<'a>>>;
    fn write_next_column<W: Write>(&mut self, writer: &mut W) -> io::Result<()>;
}

pub struct InstanceFragmentReader<'a, T: Fragment> {
    source: &'a T,
    position: (u32, u32),
}

impl<'a, T: Fragment> FragmentReader<'a, T> for InstanceFragmentReader<'a, T> {
    fn header(&self) -> FragmentHeader {
        self.source.header()
    }

    fn position(&self) -> (u32, u32) {
        self.position
    }

    fn get_next_column(&mut self) -> io::Result<Option<Column<'a>>> {
        let tables = self.source.tables();

        while (self.position.0 as usize) < tables.len() {
            let table = tables[self.position.0 as usize];
            let columns = table.columns();

            if (self.position.1 as usize) < columns.len() {
                let col = &columns[self.position.1 as usize];

                // advance column
                self.position.1 += 1;

                return Ok(Some(col.clone()));
            }

            // move to next table
            self.position.0 += 1;
            self.position.1 = 0;
        }

        Ok(None)
    }
    
    fn write_next_column<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        use Column::*;

        let tables = self.source.tables();

        while (self.position.0 as usize) < tables.len() {
            let table = tables[self.position.0 as usize];
            let columns = table.columns();

            if (self.position.1 as usize) < columns.len() {
                let col = &columns[self.position.1 as usize];

                // advance column
                self.position.1 += 1;

                match col {
                    Unsigned8(items) => {
                        writer.write_all(&[0u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Unsigned16(items) => {
                        writer.write_all(&[1u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Unsigned32(items) => {
                        writer.write_all(&[2u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Unsigned64(items) => {
                        writer.write_all(&[3u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Signed8(items) => {
                        writer.write_all(&[4u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Signed16(items) => {
                        writer.write_all(&[5u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Signed32(items) => {
                        writer.write_all(&[6u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Signed64(items) => {
                        writer.write_all(&[7u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Float32(items) => {
                        writer.write_all(&[8u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    Float64(items) => {
                        writer.write_all(&[9u8])?;
                        writer.write_all(bytemuck::cast_slice(items))?;
                    }

                    String(items) => {
                        // assuming StringColumn = offsets + data blob
                        writer.write_all(&(items.offsets.len() as u64).to_le_bytes())?;
                        writer.write_all(&[10u8])?;

                        writer.write_all(bytemuck::cast_slice(items.offsets))?;
                        writer.write_all(items.data)?;
                    }
                }

                return Ok(());
            }

            // move to next table
            self.position.0 += 1;
            self.position.1 = 0;
        }

        Ok(())
    }
}

pub struct StreamFragmentReader<'a, T: Fragment, R: Read> {
    source:    &'a mut R,
    header:    FragmentHeader,
    position:  (u32, u32),
    buffer:    Vec<u8>,
    _phantom:  PhantomData<T>,
}

impl<'a, T: Fragment, R: Read> StreamFragmentReader<'a, T, R> {
    pub fn new(source: &'a mut R) -> io::Result<Self> {
        let header = FragmentHeader::decode(source)?;
        Ok(Self { 
            source, 
            header, 
            position: (0, 0),
            buffer: Vec::new(),
            _phantom: PhantomData,
        })
    }
}

impl<'a, T: Fragment, R: Read> FragmentReader<'a, T> for StreamFragmentReader<'a, T, R> {
    fn header(&self) -> FragmentHeader {
        self.header
    }

    fn position(&self) -> (u32, u32) {
        self.position
    }

    fn get_next_column(&'a mut self) -> io::Result<Option<Column<'a>>> {
        use crate::ColumnType::*;

        let schema = T::SCHEMA;

        while (self.position.0 as usize) < schema.tables.len() {
            let table = &schema.tables[self.position.0 as usize];
            let columns = table.columns;

            if (self.position.1 as usize) < columns.len() {
                let col = &columns[self.position.1 as usize];

                // ---- read length ----
                let mut u64_buf = [0u8; 8];
                self.source.read_exact(&mut u64_buf)?;
                let len = u64::from_le_bytes(u64_buf) as usize;

                self.buffer.clear();

                match col.r#type {
                    Unsigned8 => {
                        self.buffer.resize(len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, u8>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Unsigned8(data)));
                    }

                    Unsigned16 => {
                        let byte_len = len * 2;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, u16>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Unsigned16(data)));
                    }

                    Unsigned32 => {
                        let byte_len = len * 4;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, u32>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Unsigned32(data)));
                    }

                    Unsigned64 => {
                        let byte_len = len * 8;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, u64>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Unsigned64(data)));
                    }

                    Signed8 => {
                        self.buffer.resize(len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, i8>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Signed8(data)));
                    }

                    Signed16 => {
                        let byte_len = len * 2;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, i16>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Signed16(data)));
                    }

                    Signed32 => {
                        let byte_len = len * 4;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, i32>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Signed32(data)));
                    }

                    Signed64 => {
                        let byte_len = len * 8;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, i64>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Signed64(data)));
                    }

                    Float32 => {
                        let byte_len = len * 4;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, f32>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Float32(data)));
                    }

                    Float64 => {
                        let byte_len = len * 8;
                        self.buffer.resize(byte_len, 0);
                        self.source.read_exact(&mut self.buffer)?;

                        let data = bytemuck::cast_slice::<u8, f64>(&self.buffer);
                        self.position.1 += 1;
                        return Ok(Some(Column::Float64(data)));
                    }

                    String => {
                        todo!("string decoding with offset buffer")
                    }
                }
            }

            self.position.0 += 1;
            self.position.1 = 0;
        }

        Ok(None)
    }
}