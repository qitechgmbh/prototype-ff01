use std::{collections::VecDeque, path::PathBuf};

use crate::telemetry::{binary::Fragment, segment::DataFragment};

#[derive(Debug, Clone, Default)]
pub struct ArchiveRegistry {
    pub tiers:   Vec<ArchiveTierRegistryEntry>,
    pub orphans: SubRegistry,
}

impl ArchiveRegistry {
    pub fn new(archive_dir: &PathBuf, tiers: &Vec<ArchiveTier>) -> Self {
        let tiers_raw = tiers;
        let mut tiers = Vec::new();        
        for (i, tier) in tiers_raw.iter().enumerate() {
            let path = archive_dir.join(i.to_string());
            tiers.push(ArchiveTierRegistryEntry {
                triggger: tier.triggger,
                capacity: tier.capacity,
                registry: SubRegistry::new(path, 0),
            });
        }

        let orphans = SubRegistry::new(archive_dir.join("orphans"), 128);
        Self { orphans, tiers }
    }

    pub fn tier_0(&mut self) -> &ArchiveTierRegistryEntry {
        self.tiers.get(0).expect("Must exist")
    }

    pub fn tier_0_mut(&mut self) -> &mut ArchiveTierRegistryEntry {
        self.tiers.get_mut(0).expect("Must exist")
    }
}

#[derive(Debug, Clone, Default)]
pub struct ArchiveTierRegistryEntry {
    pub triggger: u16,
    pub capacity: u16,
    pub registry: SubRegistry,
}

impl ArchiveTierRegistryEntry {
    pub fn can_promote(&self) -> bool {
        self.registry.len() >= self.triggger as usize
    }

    pub fn is_full(&self) -> bool {
        self.registry.len() >= self.triggger as usize
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ArchiveTier {
    pub triggger: u16,
    pub capacity: u16,
}

#[derive(Debug, Clone, Default)]
pub struct SubRegistry {
    path:     PathBuf,
    items:    VecDeque<SubRegistryEntry>,
    capacity: u16,

    // to add, to remove
    pending_commit: Option<(SubRegistryEntry, SubRegistryEntry)>
}

pub struct SubRegistryTransaction<'a> {
    registry: &'a mut SubRegistry,
    added:    SubRegistryEntry,
    removed:  Option<SubRegistryEntry>,
    committed: bool,
}

impl<'a> SubRegistryTransaction<'a> {
    pub fn new(
        registry: &'a mut SubRegistry,
        added: SubRegistryEntry,
        removed: Option<SubRegistryEntry>,
    ) -> Self {
        Self {
            registry,
            added,
            removed,
            committed: false,
        }
    }

    pub fn commit(mut self) {
        self.committed = true;
        // consumed → Drop still runs, but sees committed = true
    }
}

impl<'a> Drop for SubRegistryTransaction<'a> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        // --- rollback ---

        // remove the added entry (assumes it was appended)
        if let Some(last) = self.registry.items.pop() {
            debug_assert_eq!(last.uid, self.added.uid);
        }

        // restore removed entry if there was one
        if let Some(entry) = self.removed.take() {
            self.registry.items.push(entry);
        }
    }
}

impl SubRegistry {
    pub fn new<P: Into<PathBuf>>(path: P, capacity: u16) -> Self {
        Self {
            path:  path.into(),
            items: VecDeque::new(),
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &SubRegistryEntry> {
        self.items.iter()
    }

    pub fn remove_oldest(&mut self) -> Option<SubRegistryEntry> {
        self.items.pop_front()
    }

    pub fn remove_newest(&mut self) -> Option<SubRegistryEntry> {
        self.items.pop_back()
    }

    pub fn register(
        &mut self, 
        fragment: &DataFragment
    ) -> (SubRegistryEntry, Vec<SubRegistryEntry>) {
        let mut removed = Vec::new();

        while self.items.len() >= self.capacity as usize {
            let Some(entry) = self.items.pop_front() else { break; };
            removed.push(entry);
        }

        assert!(self.items.len() < self.capacity as usize);

        let (from, to) = fragment.range();

        let entry = SubRegistryEntry {
            path: self.path.join(from.to_string()),
            uid:  from,
            from,
            to,
        };

        self.items.push_back(entry.clone());

        (entry, removed)
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&SubRegistryEntry> {
        self.items.get(idx)
    }

    pub fn get_by_uid(&self, uid: u64) -> Option<&SubRegistryEntry> {
        self.items.iter().find(|e| e.uid == uid)
    }

    pub fn push(&mut self, entry: SubRegistryEntry) -> Option<SubRegistryEntry> {

        self.items.push(entry);
    }

    pub fn path_of(&self, uid: u64) -> PathBuf {
        self.path.join(uid.to_string()).join(".qta")
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubRegistryEntry {
    pub path: PathBuf,
    pub uid:  u64,
    pub from: u64,
    pub to:   u64,
}

pub struct SubRegistryGuard<'a> {
    registry: &'a mut SubRegistry,
    committed: bool,
}

impl<'a> SubRegistryGuard<'a> {
    pub fn new(registry: &'a mut SubRegistry) -> Self {
        Self {
            registry,
            committed: false,
        }
    }

    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl<'a> Drop for SubRegistryGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            self.registry.remove_newest();
        }
    }
}