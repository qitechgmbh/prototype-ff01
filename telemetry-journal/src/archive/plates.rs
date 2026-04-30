use std::{fs::File, io, path::PathBuf, sync::Arc};

use arrow::{
    array::{ArrayRef, Int16Builder, RecordBatch, TimestampNanosecondBuilder}, 
    datatypes::{DataType, Field, Schema, TimeUnit}
};
use parquet::arrow::ArrowWriter;

use crate::PlateEvent;

#[derive(Debug, Default)]
pub struct PlatesArchive {
    ts:   TimestampNanosecondBuilder,
    peak: Int16Builder,
    real: Int16Builder,
}

impl PlatesArchive {
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
            Field::new("peak", DataType::Int16, false),
            Field::new("real", DataType::Int16, false),
        ]))
    }

    pub fn push(&mut self, ts: u64, event: PlateEvent) {
        self.ts.append_value(ts as i64);
        self.peak.append_value(event.peak);
        self.real.append_value(event.real);
    }

    pub fn export(mut self, path: &PathBuf) -> io::Result<()> {
        use io::Error;
        use io::ErrorKind;

        let schema = Self::schema();

        let ts_array:   ArrayRef = Arc::new(self.ts.finish());
        let peak_array: ArrayRef = Arc::new(self.peak.finish());
        let real_array: ArrayRef = Arc::new(self.real.finish());

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![ts_array, peak_array, real_array],
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