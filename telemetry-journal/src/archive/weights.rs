use std::{fs::File, io, path::PathBuf, sync::Arc};

use arrow::{
    array::{ArrayRef, Int16Builder, RecordBatch, TimestampNanosecondBuilder}, 
    datatypes::{DataType, Field, Schema, TimeUnit}
};
use parquet::arrow::ArrowWriter;

use telemetry_core::WeightEvent;

#[derive(Debug, Default)]
pub struct WeightsArchive {
    ts: TimestampNanosecondBuilder,
    weight_0: Int16Builder,
    weight_1: Int16Builder,
}

impl WeightsArchive {
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
            Field::new("w0", DataType::Int16, true),
            Field::new("w1", DataType::Int16, true),
        ]))
    }

    pub fn push(&mut self, ts: u64, event: WeightEvent) {
        self.ts.append_value(ts as i64);
        self.weight_0.append_option(event.weight_0);
        self.weight_1.append_option(event.weight_1);
    }

    pub fn export(mut self, path: &PathBuf) -> io::Result<()> {
        use io::Error;
        use io::ErrorKind;

        let schema = Self::schema();

        let ts_array: ArrayRef = Arc::new(self.ts.finish());
        let w0_array: ArrayRef = Arc::new(self.weight_0.finish());
        let w1_array: ArrayRef = Arc::new(self.weight_1.finish());

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![ts_array, w0_array, w1_array],
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