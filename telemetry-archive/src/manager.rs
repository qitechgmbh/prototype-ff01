use std::{fs::create_dir_all, io, path::PathBuf};

use heapless::Deque;

use crate::{ArchiveTier, FragmentSchema};

#[derive(Debug)]
pub struct ArchiveManager {
    schema:   &'static FragmentSchema,
    tiers:    &'static [ArchiveTier],
    registry: Deque<RegistryEntry, 256>,
}

// manager opens temporary file and exposes an archive writer
// user can then append to that archive and finish with a 
// ArchiveCreateTranscation::commit()

impl ArchiveManager {
    pub fn new(
        root:   PathBuf, 
        schema: &'static FragmentSchema,
        tiers:  &'static [ArchiveTier],
    ) -> io::Result<Self> {
        if config.tiers.len() == 0 {
            panic!("Requires atleast one tier");
        }

        create_dir_all(config.root)?;

        
    }

    pub fn add_archive() {
        
    }
}

#[derive(Debug)]
pub struct Registry { 
    entries: Deque<RegistryEntry, 256>,
}

#[derive(Debug)]
pub struct RegistryEntry {
    pub name: heapless::String<40>,
    pub tier: u16,
    pub from: u64,
    pub to:   u64,
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf};

    use crate::{ArchiveTier, manager::{ArchiveManager}};

    #[test]
    fn parse_static_schema() {
        let archive_dir = PathBuf::from("/home/entity/sandbox/ff01/machine/telemetry");
       
        let mut tiers = heapless::Vec::new();
        tiers.push(ArchiveTier {
            capacity_desired: 5,
            capacity_max:     10,
        });

        let config = Config {
            root: archive_dir,
            tiers: tiers,
        };

        let manager = ArchiveManager::new(config);

        println!("{:?}", root);
    }
}