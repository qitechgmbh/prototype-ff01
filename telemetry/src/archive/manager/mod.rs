use std::{fs::{File, create_dir_all}, io, path::PathBuf, sync::Arc};

use crossbeam::channel::Receiver;

use crate::{Shared, archive::{ArchiveTier, Fragment, FragmentBody, Writer}};

mod tier_registry;
pub use tier_registry::TierRegistry;
pub use tier_registry::TierEntry;
pub use tier_registry::TierPromoteTranscation;

mod fragment_registry;
pub use fragment_registry::FragmentRegistry;
pub use fragment_registry::FragmentRegistryEntry;
pub use fragment_registry::AppendTransaction;

#[derive(Debug)]
pub struct Config {
    pub archive_dir: PathBuf,
    pub tiers:       Vec<ArchiveTier>,
}

#[derive(Debug)]
pub struct ArchiveManager<Body: FragmentBody> {
    shared:      Arc<Shared<Body>>,
    fragment_rx: Receiver<Fragment<Body>>,
    registry:    TierRegistry,
    tmp_dir:     PathBuf,
}

impl<Body: FragmentBody> ArchiveManager<Body> {
    pub(crate) fn new(
        config: Config, 
        shared: Arc<Shared<Body>>, 
        segment_rx: Receiver<Fragment<Body>>
    ) -> Result<Self, io::Error> {
        if config.tiers.len() == 0 {
            panic!("Requires atleast one tier");
        }

        let registry = TierRegistry::new(&config.archive_dir, &config.tiers);

        Ok(Self { 
            shared, 
            fragment_rx: segment_rx, 
            registry,
            tmp_dir: config.archive_dir.join("tmp")
        })
    }

    pub fn run(mut self) -> io::Result<()> {
        _ = self.shared;

        loop {
            match self.fragment_rx.recv() {
                Ok(segment) => {
                    println!("Received fragment");
                    self.flush_fragment(&segment)?;
                    self.sync_registry();
                    // self.promote_all()?;
                }

                Err(_) => {
                    return Ok(()); // channel closed → shutdown cleanly
                }
            }
        }
    }

    /*
    fn reload_registry(&mut self) -> io::Result<()> {
        // clear current in-memory state
        self.registry.orphans.items.clear();

        // ---- reload orphans ----
        for entry in fs::read_dir(&self.registry.orphans.path)? {
            let entry = entry?;
            let path  = entry.path();

            if !path.is_file() {
                continue;
            }

            // adapt this parsing to your actual encoding scheme
            let uid = parse_uid_from_path(&path);

            self.registry.orphans.create_append_transaction(from, to);
            
            .push_back(FragmentRegistryEntry {
                uid,
                from: 0,
                to: 0,
                path,
            });
        }

        // ---- reload tiers ----
        for tier in self.registry.tiers.iter_mut() {
            tier.registry.items.clear();

            for entry in fs::read_dir(&tier.registry.path)? {
                let entry = entry?;
                let path = entry.path();

                if !path.is_file() {
                    continue;
                }

                let uid = parse_uid_from_path(&path);

                tier.registry.items.push(TierRegistryEntry {
                    uid,
                    path: path.clone(),
                });
            }
        }

        Ok(())
    }
    */

    pub fn flush_fragment(&mut self, fragment: &Fragment<Body>) -> io::Result<()> {
        let tier = self.registry.tier_0_mut();

        let metadata = fragment.metadata();
        let transaction = tier.registry.create_append_transaction(
            metadata.from, 
            metadata.to,
        );

        let tmp_path = self.tmp_dir.join("segment.tmp");
        let out_path = transaction.added_entry().path.clone();

        create_dir_all(tmp_path.parent().unwrap())?;
        create_dir_all(out_path.parent().unwrap())?;

        let mut file = File::open(tmp_path)?;

        // TODO: sync issue when archive written but deletion fails
        // now registry is out of sync!!
        let mut archive = Writer::new(&mut file)?;
        archive.write_fragment(&fragment)?;

        if let Some(entry) = transaction.removed_entry() {
            println!("Removing: {:?}", entry.uid);
            std::fs::remove_file(&entry.path)?;
        }

        transaction.commit();

        Ok(())
    }

    fn sync_registry(&mut self) {
        let registry = self.registry.clone();
        self.shared.tier_registry.store(Arc::new(registry));
    }

        /* 
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
    */
}