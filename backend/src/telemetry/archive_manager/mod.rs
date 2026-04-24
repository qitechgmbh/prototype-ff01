use std::{fs::create_dir_all, io, path::PathBuf, sync::Arc};

use crossbeam::channel::Receiver;

use crate::telemetry::{
    Shared, 
    archive_manager::types::{SubRegistryGuard}, 
    binary::{self, ArchiveReader}, 
    segment::DataFragment
};

// mod types;
// pub use types::ArchiveRegistry;
// pub use types::ArchiveTier;

mod archive_registry;
mod fragment_registry;

#[derive(Debug)]
pub struct Config {
    pub archive_dir: PathBuf,
    pub tiers:       Vec<ArchiveTier>,
}

#[derive(Debug)]
pub struct ArchiveManager {
    shared:     Arc<Shared>,
    segment_rx: Receiver<Box<DataFragment>>,
    registry:   ArchiveRegistry,
    tmp_dir:    PathBuf,
}

impl ArchiveManager {
    pub fn new(
        config: Config, 
        shared: Arc<Shared>, 
        segment_rx: Receiver<Box<DataFragment>>
    ) -> Result<Self, io::Error> {
        if config.tiers.len() == 0 {
            panic!("Requires atleast one tier");
        }

        let registry = ArchiveRegistry::new(&config.archive_dir, &config.tiers);

        Ok(Self { 
            shared, 
            segment_rx, 
            registry,
            tmp_dir: config.archive_dir.join("tmp")
        })
    }

    pub fn run(mut self) -> io::Result<()> {
        _ = self.shared;

        loop {
            match self.segment_rx.recv() {
                Ok(segment) => {
                    println!("Received fragment");
                    self.flush_fragment(segment)?;
                    self.sync_registry();
                    self.promote_all()?;
                }

                Err(_) => {
                    return Ok(()); // channel closed → shutdown cleanly
                }
            }
        }
    }

    pub fn flush_fragment(&mut self, fragment: Box<DataFragment>) -> io::Result<()> {
        let tier = self.registry.tier_0_mut();

        let (entry, removed_entries) = tier.registry.register(&fragment);

        let guard = SubRegistryGuard::new(&mut tier.registry);

        let tmp_path = self.tmp_dir.join("segment.tmp");
        let out_path = entry.path.clone();

        create_dir_all(out_path.parent().unwrap())?;
        create_dir_all(tmp_path.parent().unwrap())?;

        let mut archive = binary::ArchiveWriter::create(
            tmp_path,
            out_path,
            binary::MAGIC, 
            binary::VERSION
        )?;

        archive.write_fragment(&*fragment)?;
        archive.finalize()?;

        // success → commit
        guard.commit();

        // successfully commited. Delete overflow entries
        for entry in removed_entries {
            _ = std::fs::remove_file(entry.path);
        }

        Ok(())
    }

    fn sync_registry(&mut self) {
        let registry = self.registry.clone();
        self.shared.segment_registry.store(Arc::new(registry));
    }

    fn promote_all(&mut self) -> io::Result<()>  {
        let timestamp = 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();

        for (i, tier) in self.registry.tiers.iter().enumerate() {
            println!("Goal: {:?} | {:?}", tier, i);

            if !tier.can_promote() {
                return Ok(());
            }

            let first = tier.registry.get_by_index(0).expect("Must have atleast one");

            let tmp_path = self.tmp_dir.join("merge.tmp");
            let out_path = tier.registry.path_of(0);
            create_dir_all(&out_path.parent().unwrap())?;
            create_dir_all(&tmp_path.parent().unwrap())?;

            let mut archive = binary::ArchiveWriter::create(
                tmp_path,
                out_path,
                binary::MAGIC, 
                binary::VERSION
            )?;

            for entry in tier.registry.iter() {
                let file_path = tier.registry.path_of(entry.uid);

                let mut reader = ArchiveReader::open(&file_path)?;

                loop {
                    let Some(fragment) = reader.next::<DataFragment>()? else { break; };
                    archive.write_fragment(&fragment)?;
                }
            }

            archive.finalize()?;
        }

        Ok(())
    }
}