use std::{fs::File, io, path::PathBuf, sync::Arc};

use arrow::{
    array::{ArrayRef, RecordBatch, StringBuilder, TimestampNanosecondBuilder, UInt8Builder}, 
    datatypes::{DataType, Field, Schema, TimeUnit}
};
use parquet::arrow::ArrowWriter;

use crate::LogEvent;

#[derive(Debug, Default)]
pub struct LogsArchive {
    ts:       TimestampNanosecondBuilder,
    category: UInt8Builder,
    message:  StringBuilder,
}

impl LogsArchive {
    pub fn schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new(
                "ts",
                DataType::Timestamp(
                    TimeUnit::Nanosecond,
                    None,
                ),
                false,
            ),
            Field::new("category", DataType::UInt8, false),
            Field::new("message",  DataType::Utf8,  false),
        ]))
    }

    pub fn push(&mut self, ts: u64, event: LogEvent) {
        self.ts.append_value(ts as i64);
        self.category.append_value(event.category as u8);
        self.message.append_value(event.message);
    }

    pub fn export(mut self, path: &PathBuf) -> io::Result<()> {
        use io::Error;
        use io::ErrorKind;

        let schema = Self::schema();

        let ts_array:       ArrayRef = Arc::new(self.ts.finish());
        let category_array: ArrayRef = Arc::new(self.category.finish());
        let message_array:  ArrayRef = Arc::new(self.message.finish());

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![ts_array, category_array, message_array],
        ).map_err(|e| {
            println!("Batch Error: {}", e);
            Error::new(ErrorKind::Other, "batch error")
        })?;
        
        println!("creating: {:?}", &path);
        let file = File::create(path)?;

        let mut writer = ArrowWriter::try_new(file, schema, None)
            .map_err(|_| Error::new(ErrorKind::Other, "writer error"))?;

        writer.write(&batch)
            .map_err(|_| Error::new(ErrorKind::Other, "write error"))?;

        writer.close()
            .map_err(|_| Error::new(ErrorKind::Other, "close error"))?;

        Ok(())
    }
}