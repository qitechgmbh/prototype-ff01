use std::{
    fs::File, io, path::PathBuf
};

use crate::telemetry::binary::Batch;

pub fn save_batch(archive_dir: PathBuf, batch: &Batch) -> io::Result<()> {
    let secs = batch.secs_since_ue();
    let filename = format!("{}.qts", secs);

    let mut path = archive_dir;
    path.push(filename);

    println!("{:?}", path);

    let mut file = File::create(path).unwrap();
    batch.write(&mut file)
}