use std::io::{self, Read, Seek, Write};

use crate::{Column, Fragment, MAGIC, VERSION, io::ArchiveHeader};

#[derive(Debug)]
pub struct FragmentStitcher<'a, I: Read, O: Write + Read + Seek> {
    header: ArchiveHeader,
    input_stream: heapless::Vec<&'a mut I, 32>,
    output_streams: &'a mut O,
}

pub fn merge_fragments<I, O>(inputs: &[&I], output: &mut O) -> io::Result<()>
where
    I: Fragment,
    O: Write + Read + Seek,
{
    use Column::*;

    for input in inputs {
        for table in input.tables() {
            for column in table.columns() {
                match column {
                    Column::Unsigned8(items) => {
                        output.write_all(items)?;
                    }

                    Column::Unsigned16(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Unsigned32(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Unsigned64(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Signed8(items) => {
                        let bytes = unsafe {
                            std::slice::from_raw_parts(
                                items.as_ptr() as *const u8,
                                items.len(),
                            )
                        };
                        output.write_all(bytes)?;
                    }

                    Column::Signed16(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Signed32(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Signed64(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Float32(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::Float64(items) => {
                        for v in *items {
                            output.write_all(&v.to_le_bytes())?;
                        }
                    }

                    Column::String(items) => {
                        for s in *items {
                            let bytes = s.as_bytes();
                            let len = bytes.len() as u32;

                            output.write_all(&len.to_le_bytes())?;
                            output.write_all(bytes)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}



impl<'a, I, O> FragmentStitcher<'a, I, O> 
where
    I: Read,
    O: Write + Read + Seek,
  {
    pub fn new(
        input_stream: heapless::Vec<&'a mut I, 32>, 
        output_streams: &'a mut O
    ) -> io::Result<Self> {
        let header = ArchiveHeader {
            magic:          MAGIC,
            version:        VERSION,
            fragment_count: 0,
            data_size:      0,
        };
        
        header.write(output_streams)?;
        Ok(Self { header, input_stream, output_streams })
    }

    
}