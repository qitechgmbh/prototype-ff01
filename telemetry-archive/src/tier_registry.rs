use std::{collections::vec_deque, iter::Take};

use crate::{ArchiveTier, fragment_registry::{FragmentRegistry, FragmentRegistryEntry}};

#[derive(Debug, Clone, Default)]
pub struct TierRegistry {
    pub tiers: Vec<TierEntry>,
    pub orphans: FragmentRegistry,
}

impl TierRegistry {
    pub fn new(tiers: &Vec<ArchiveTier>) -> Self {
        let tiers_raw = tiers;
        let mut tiers = Vec::new();
        for tier in tiers_raw.iter() {
            tiers.push(TierEntry {
                tier: *tier,
                registry: FragmentRegistry::new(tier.capacity_max),
            });
        }

        let orphans = FragmentRegistry::new(128);
        Self { orphans, tiers }
    }

    pub fn tier_0(&mut self) -> &TierEntry {
        self.tiers.get(0).expect("Must exist")
    }

    pub fn tier_0_mut(&mut self) -> &mut TierEntry {
        self.tiers.get_mut(0).expect("Must exist")
    }
}

#[derive(Debug, Clone, Default)]
pub struct TierEntry {
    pub tier:     ArchiveTier,
    pub registry: FragmentRegistry,
}

impl TierEntry {
    pub fn can_promote(&self) -> bool {
        self.registry.len() >= self.tier.capacity_desired
    }

    pub fn is_full(&self) -> bool {
        self.registry.len() >= self.tier.capacity_max
    }
}

#[derive(Debug)]
pub struct TierPromoteTranscation<'a> {
    src_tier: &'a mut FragmentRegistry,
    dest_tier: &'a mut FragmentRegistry,
    transfer_amount: usize,
}

impl<'a> TierPromoteTranscation<'a> {
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
            uuid: 999,
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
