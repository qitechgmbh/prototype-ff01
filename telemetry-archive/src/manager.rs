use std::{io, path::PathBuf, sync::Arc};

use crate::{ArchiveTier, FragmentSchema, tier_registry::TierRegistry};

#[derive(Debug)]
pub struct Config {
    pub archive_dir: PathBuf,
    pub tiers:       heapless::Vec<ArchiveTier, 32>,
}

#[derive(Debug)]
pub struct ArchiveManager {
    schema:    FragmentSchema,
    registry:  TierRegistry,
    tmp_dir:   PathBuf,
}

impl ArchiveManager {
    pub fn new(config: Config) -> io::Result<Self> {
        if config.tiers.len() == 0 {
            panic!("Requires atleast one tier");
        }

        let registry = TierRegistry {
            
        };
    }
}