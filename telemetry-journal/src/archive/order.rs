use std::{fs::File, io, path::PathBuf, sync::Arc};

use parquet::arrow::ArrowWriter;

use arrow::{
    array::{
        ArrayRef, 
        BooleanBuilder, 
        RecordBatch, 
        TimestampNanosecondBuilder, 
        UInt16Builder, 
        UInt32Builder
    }, 
    datatypes::{
        DataType, 
        Field, 
        Schema, 
        TimeUnit
    }
};

use crate::{OrderEvent};

#[derive(Debug, Default)]
pub struct OrdersArchive {
    ts:          TimestampNanosecondBuilder,
    started:     BooleanBuilder,
    order_id:    UInt32Builder,
    worker_id:   UInt32Builder,
    scrap_count: UInt16Builder,
}

impl OrdersArchive {
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
            Field::new("started",     DataType::Boolean, false),
            Field::new("order_id",    DataType::UInt32,  false),
            Field::new("worker_id",   DataType::UInt32,  true),
            Field::new("scrap_count", DataType::UInt16,  true),
        ]))
    }

    pub fn push(&mut self, ts: u64, event: OrderEvent) {
        self.ts.append_value(ts as i64);
        self.started.append_value(event.started);
        self.order_id.append_value(event.order_id);
        self.worker_id.append_option(event.worker_id);
        self.scrap_count.append_option(event.scrap_count);
    }

    pub fn export(mut self, path: &PathBuf) -> io::Result<()> {
        use io::Error;
        use io::ErrorKind;

        let schema = Self::schema();

        let ts_array:          ArrayRef = Arc::new(self.ts.finish());
        let started_array:     ArrayRef = Arc::new(self.started.finish());
        let order_id_array:    ArrayRef = Arc::new(self.order_id.finish());
        let worker_id_array:   ArrayRef = Arc::new(self.worker_id.finish());
        let scrap_count_array: ArrayRef = Arc::new(self.scrap_count.finish());

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                ts_array, 
                started_array, 
                order_id_array, 
                worker_id_array, 
                scrap_count_array
            ],
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