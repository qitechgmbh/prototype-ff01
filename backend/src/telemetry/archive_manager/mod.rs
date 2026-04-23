use std::{fs::{File, create_dir_all}, io, path::PathBuf, sync::Arc};

use crossbeam::channel::Receiver;

use crate::telemetry::{Shared, binary::{self, ArchiveReader, ArchiveWriter, MAGIC, VERSION}, segment::DataFragment};

mod types;
pub use types::ArchiveRegistry;

#[derive(Debug)]
pub struct Config {
    pub archive_dir: PathBuf,
    pub tiers:       Vec<u16>,
}

#[derive(Debug)]
pub struct ArchiveManager {
    config:     Config,
    shared:     Arc<Shared>,
    segment_rx: Receiver<Box<DataFragment>>,
    registry:   ArchiveRegistry,
    tiers:      Vec<(PathBuf, u16)>,
    orphan_dir: PathBuf,
}

impl ArchiveManager {
    pub fn new(
        config: Config, 
        shared: Arc<Shared>, 
        segment_rx: Receiver<Box<DataFragment>>
    ) -> Result<Self, io::Error> {
        create_dir_all(&config.archive_dir)?;

        let orphan_dir = config.archive_dir.join("orphans");
        println!("Creating {:?}", &orphan_dir);
        create_dir_all(&orphan_dir)?;

        let mut tiers: Vec<(PathBuf, u16)> = Vec::new();

        let level0_dir = config.archive_dir.join("tier_0");
        println!("Creating {:?}", &level0_dir);
        create_dir_all(&level0_dir)?;
        tiers.push((level0_dir, 0));

        let mut i: u16 = 1;
        for tier_capacity in &config.tiers {
            let extension = format!("tier_{}", i);
            let dir = config.archive_dir.join(extension);
            println!("Creating {:?}", &dir);
            create_dir_all(&dir)?;
            tiers.push((dir, *tier_capacity));
            i += 1;
        }

        let tier_count = config.tiers.len();

        Ok(Self { 
            config, 
            shared, 
            segment_rx, 
            orphan_dir, 
            tiers,
            registry: ArchiveRegistry::new(tier_count),
        })
    }

    pub fn run(mut self) {
        _ = self.config;
        _ = self.shared;
        _ = self.orphan_dir;
        _ = self.tiers;

        loop {
            match self.segment_rx.recv() {
                Ok(segment) => {
                    self.flush_segment(segment);
                    self.sync_registry();
                    self.try_shrink_to_level(1);
                }

                Err(_) => {
                    break; // channel closed → shutdown cleanly
                }
            }
        }
    }

    pub fn flush_segment(&mut self, fragment: Box<DataFragment>) {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let name = format!("{}.qts", secs);
        let path = self.tier_0().0.join(name);

        let mut archive = binary::ArchiveWriter::create(path, binary::MAGIC, binary::VERSION).unwrap();
        archive.write_fragment(&*fragment).unwrap();
        archive.finalize().unwrap();

        self.registry.tier_0_mut().push((secs, 0, 0));
    }

    fn tier_0(&self) -> &(PathBuf, u16) {
        self.tiers.get(0).expect("Must be present")
    }

    fn sync_registry(&mut self) {
        let registry = self.registry.clone();
        self.shared.segment_registry.store(Arc::new(registry));
    }

    fn try_shrink_to_level(&mut self, tier_idx: usize) {
        println!("SHRINKING");

        let Some(registry) = self.registry.tiers.get(tier_idx) else { return; };

        let tier = &self.tiers[tier_idx];
        if registry.len() < tier.1 as usize {
            return;
        }

        let tmp_dir = self.config.archive_dir.join("tmp");
        create_dir_all(&tmp_dir).unwrap();

        let tmp_path = tmp_dir.join(format!(
            "merge_{}.qts",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));

        let mut writer = ArchiveWriter::create(tmp_path, MAGIC, VERSION).unwrap();

        for (ts, _, _) in registry.iter() {
            let file_path = tier.0.join(format!("{}.qts", ts));

            let mut reader = ArchiveReader::open(&file_path).unwrap();

            loop {
                let Some(fragment) = reader.next::<DataFragment>().unwrap() else { break; };
                writer.write_fragment(&fragment).unwrap();
            }
        }

        writer.finalize().unwrap();
    }
}