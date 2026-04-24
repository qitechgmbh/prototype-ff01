use std::{collections::vec_deque, iter::Take, path::PathBuf};

use crate::telemetry::{
    archive_manager::{
        ArchiveTier,
        fragment_registry::{FragmentRegistry, FragmentRegistryEntry},
    },
};

#[derive(Debug, Clone, Default)]
pub struct TierRegistry {
    pub tiers: Vec<ArchiveEntry>,
    pub orphans: FragmentRegistry,
}

impl TierRegistry {
    pub fn new(archive_dir: &PathBuf, tiers: &Vec<ArchiveTier>) -> Self {
        let tiers_raw = tiers;
        let mut tiers = Vec::new();
        for (i, tier) in tiers_raw.iter().enumerate() {
            let path = archive_dir.join(i.to_string());
            tiers.push(ArchiveEntry {
                triggger: tier.triggger,
                capacity: tier.capacity,
                registry: FragmentRegistry::new(path, 0),
            });
        }

        let orphans = FragmentRegistry::new(archive_dir.join("orphans"), 128);
        Self { orphans, tiers }
    }

    pub fn tier_0(&mut self) -> &ArchiveEntry {
        self.tiers.get(0).expect("Must exist")
    }

    pub fn tier_0_mut(&mut self) -> &mut ArchiveEntry {
        self.tiers.get_mut(0).expect("Must exist")
    }
}

#[derive(Debug, Clone, Default)]
pub struct ArchiveEntry {
    pub triggger: u16,
    pub capacity: u16,
    pub registry: FragmentRegistry,
}

impl ArchiveEntry {
    pub fn can_promote(&self) -> bool {
        self.registry.len() >= self.triggger as usize
    }

    pub fn is_full(&self) -> bool {
        self.registry.len() >= self.triggger as usize
    }
}

#[derive(Debug)]
pub struct ArchivePromoteTranscation<'a> {
    src_tier: &'a mut FragmentRegistry,
    dest_tier: &'a mut FragmentRegistry,
    transfer_amount: usize,
}

impl<'a> ArchivePromoteTranscation<'a> {
    pub fn old_entries(&self) -> Take<vec_deque::Iter<'_, FragmentRegistryEntry>> {
        self.src_tier.items.iter().take(self.transfer_amount)
    }

    pub fn new_entry(&self) -> FragmentRegistryEntry {
        let first = self.src_tier.items.get(0).expect("Must have item");
        let last = self
            .src_tier
            .items
            .get(self.transfer_amount)
            .expect("Must have item");
        let from = first.from;
        let to = last.to;
        return FragmentRegistryEntry {
            path: self.dest_tier.path.join(from.to_string()),
            uid: from,
            from,
            to,
        };
    }

    pub fn commit(self) {
        for _ in 0..self.transfer_amount {
            self.src_tier.items.pop_front();
        }

        let entry = self.new_entry();
        self.dest_tier.items.push_back(entry);
    }
}
