use std::{borrow::Cow, io::{self, Read, Write}};

#[derive(Debug, Clone, Copy)]
pub enum BufferType {
    Timestamp,
    Unsigned8,
    Unsigned16,
    Unsigned32,
    Unsigned64,
    Signed8,
    Signed16,
    Signed32,
    Signed64,
    Float32,
    Float64,
    String,
}

pub type TableLayout = [BufferType];

#[derive(Debug, Clone)]
pub enum Buffer<'a> {
    Timestamp(Cow<'a, [u64]>),

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

impl<'a> Buffer<'a> {
    pub fn write<W: Write>(self, w: &mut W) -> io::Result<()> {
        match self {
            Buffer::Timestamp(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Unsigned8(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Unsigned16(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Unsigned32(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Unsigned64(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Signed8(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Signed16(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Signed32(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Signed64(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Float32(v) => {
                write_slice(w, &v)?;
            }

            Buffer::Float64(v) => {
                write_slice(w, &v)?;
            }

            Buffer::String(v) => {
                let len = v.len() as u64;
                w.write_all(&len.to_le_bytes())?;

                for s in v {
                    let bytes = s.as_bytes();
                    let slen = bytes.len() as u64;
                    w.write_all(&slen.to_le_bytes())?;
                    w.write_all(bytes)?;
                }
            }
        }

        Ok(())
    }

    pub fn read<R: Read>(
        r: &mut R,
        ty: BufferType,
        len: usize,
    ) -> io::Result<Self> {
        match ty {
            BufferType::Timestamp => {
                let v = read_vec::<u64, R>(r, len)?;
                Ok(Buffer::Timestamp(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Unsigned8 => {
                let v = read_vec::<u8, R>(r, len)?;
                Ok(Buffer::Unsigned8(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Unsigned16 => {
                let v = read_vec::<u16, R>(r, len)?;
                Ok(Buffer::Unsigned16(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Unsigned32 => {
                let v = read_vec::<u32, R>(r, len)?;
                Ok(Buffer::Unsigned32(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Unsigned64 => {
                let v = read_vec::<u64, R>(r, len)?;
                Ok(Buffer::Unsigned64(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Signed8 => {
                let v = read_vec::<i8, R>(r, len)?;
                Ok(Buffer::Signed8(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Signed16 => {
                let v = read_vec::<i16, R>(r, len)?;
                Ok(Buffer::Signed16(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Signed32 => {
                let v = read_vec::<i32, R>(r, len)?;
                Ok(Buffer::Signed32(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Signed64 => {
                let v = read_vec::<i64, R>(r, len)?;
                Ok(Buffer::Signed64(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Float32 => {
                let v = read_vec::<f32, R>(r, len)?;
                Ok(Buffer::Float32(Box::leak(v.into_boxed_slice())))
            }

            BufferType::Float64 => {
                let v = read_vec::<f64, R>(r, len)?;
                Ok(Buffer::Float64(Box::leak(v.into_boxed_slice())))
            }

            BufferType::String => {
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    let slen = read_u64(r)? as usize;
                    let mut buf = vec![0u8; slen];
                    r.read_exact(&mut buf)?;
                    out.push(String::from_utf8(buf).unwrap());
                }

                Ok(Buffer::String(Box::leak(out.into_boxed_slice())))
            }
        }
    }
}

fn write_slice<W: Write, T: bytemuck::Pod>(w: &mut W, slice: &[T]) -> io::Result<()> {
    let len = slice.len() as u64;
    w.write_all(&len.to_le_bytes())?;

    let bytes = unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * std::mem::size_of::<T>(),
        )
    };

    w.write_all(bytes)?;
    Ok(())
}

pub fn read_vec<T: Copy + Default, R: Read>(
    r: &mut R,
    len: usize,
) -> io::Result<Vec<T>> {
    let mut vec = vec![T::default(); len];

    let byte_slice = unsafe {
        std::slice::from_raw_parts_mut(
            vec.as_mut_ptr() as *mut u8,
            len * std::mem::size_of::<T>(),
        )
    };

    r.read_exact(byte_slice)?;

    Ok(vec)
}