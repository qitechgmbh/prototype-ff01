use std::io::{self, Read};

use crate::TableSchema;

pub trait Table {
    const SCHEMA: TableSchema;
    type Item;

    fn append(&mut self, ts: u64, item: Self::Item);
}

pub trait TableDyn {
    fn schema(&self) -> &'static TableSchema;
    fn len(&self) -> u64;
    fn range(&self) -> (u64, u64);
    fn columns(&self) -> &[Column];
}

#[derive(Debug, Clone)]
pub enum Column<'a> {
    Unsigned8(&'a [u8]),
    Unsigned16(&'a [u16]),
    Unsigned32(&'a [u32]),
    Unsigned64(&'a [u64]),

    Signed8(&'a [i8]),
    Signed16(&'a [i16]),
    Signed32(&'a [i32]),
    Signed64(&'a [i64]),

    Float32(&'a [f32]),
    Float64(&'a [f64]),

    String(&'a StringColumn),
}

impl<'a> Column<'a> {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Column<'a>> {
        let mut u64_buf = [0u8; 8];
        let mut u8_buf  = [0u8; 1];

        // length (elements)
        reader.read_exact(&mut u64_buf)?;
        let len = u64::from_le_bytes(u64_buf) as usize;

        // type tag
        reader.read_exact(&mut u8_buf)?;
        let ty = u8_buf[0];

        match ty {
            0 => {
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                Ok(Column::Unsigned8(Box::leak(buf.into_boxed_slice())))
            }

            1 => {
                let mut buf = vec![0u8; len * 2];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const u16;
                let len = buf.len() / 2;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut u16, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Unsigned16(Box::leak(v.into_boxed_slice())))
            }

            2 => {
                let mut buf = vec![0u8; len * 4];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const u32;
                let len = buf.len() / 4;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut u32, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Unsigned32(Box::leak(v.into_boxed_slice())))
            }

            3 => {
                let mut buf = vec![0u8; len * 8];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const u64;
                let len = buf.len() / 8;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut u64, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Unsigned64(Box::leak(v.into_boxed_slice())))
            }

            4 => {
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                Ok(Column::Signed8(Box::leak(buf.into_boxed_slice() as Box<[i8]>)))
            }

            5 => {
                let mut buf = vec![0u8; len * 2];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const i16;
                let len = buf.len() / 2;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut i16, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Signed16(Box::leak(v.into_boxed_slice())))
            }

            6 => {
                let mut buf = vec![0u8; len * 4];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const i32;
                let len = buf.len() / 4;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut i32, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Signed32(Box::leak(v.into_boxed_slice())))
            }

            7 => {
                let mut buf = vec![0u8; len * 8];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const i64;
                let len = buf.len() / 8;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut i64, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Signed64(Box::leak(v.into_boxed_slice())))
            }

            8 => {
                let mut buf = vec![0u8; len * 4];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const f32;
                let len = buf.len() / 4;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut f32, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Float32(Box::leak(v.into_boxed_slice())))
            }

            9 => {
                let mut buf = vec![0u8; len * 8];
                reader.read_exact(&mut buf)?;

                let ptr = buf.as_ptr() as *const f64;
                let len = buf.len() / 8;

                let v = unsafe {
                    Vec::from_raw_parts(ptr as *mut f64, len, len)
                };

                std::mem::forget(buf);
                Ok(Column::Float64(Box::leak(v.into_boxed_slice())))
            }

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown column type",
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StringColumn {
    data: Vec<u8>,
    offsets: Vec<u32>,
}

impl StringColumn {
    pub fn push(&mut self, s: &str) {
        self.offsets.push(self.data.len() as u32);
        self.data.extend_from_slice(s.as_bytes());
    }

    pub fn get(&self, i: usize) -> &str {
        let start = self.offsets[i] as usize;
        let end = if i + 1 < self.offsets.len() {
            self.offsets[i + 1] as usize
        } else {
            self.data.len()
        };

        std::str::from_utf8(&self.data[start..end]).unwrap()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}