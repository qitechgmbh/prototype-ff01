use std::{fs::create_dir_all, io, path::PathBuf};

use heapless::Deque;

use crate::{ArchiveTier, FragmentSchema};

#[derive(Debug)]
pub struct ArchiveManager {
    schema:   &'static FragmentSchema,
    tiers:    &'static [ArchiveTier],
    root:     PathBuf,
    registry: Deque<RegistryEntry, 256>,
}

impl ArchiveManager {
    pub fn new(
        schema: &'static FragmentSchema,
        tiers:  &'static [ArchiveTier],
        root:   PathBuf, 
    ) -> io::Result<Self> {
        assert!(tiers.len() > 0);

        create_dir_all(&root)?;

        for i in 0..tiers.len() {
            let name = format!("tier_{}", i);
            let path = root.join(name);
            create_dir_all(path)?;
        }

        // TODO: scan all dirs

        Ok(Self { schema, tiers, root, registry: Default::default() })
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
    pub tier: u8,
    pub range: (u64, u64),
}