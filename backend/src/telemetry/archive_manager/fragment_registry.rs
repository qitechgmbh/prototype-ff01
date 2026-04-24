use std::{collections::VecDeque, path::PathBuf};

use crate::telemetry::{binary::Fragment, segment::DataFragment};

#[derive(Clone, Debug, Default)]
pub struct FragmentRegistry {
    pub(crate) path:  PathBuf,
    pub(crate) items: VecDeque<FragmentRegistryEntry>,
}

impl FragmentRegistry {
    pub fn new<P: Into<PathBuf>>(path: P, capacity: u16) -> Self {
        Self {
            path: path.into(),
            items: VecDeque::with_capacity(capacity as usize),
        }
    }

    pub fn begin_transaction(
        &mut self,
        fragment: &DataFragment
    ) -> FragmentRegistryTransaction<'_> {
        let (from, to) = fragment.range();
        
        let entry = FragmentRegistryEntry {
            path: self.path.join(from.to_string()),
            uid:  from,
            from,
            to,
        };

        let should_remove = self.items.len() == self.items.capacity();

        FragmentRegistryTransaction {
            registry: self,
            added:    entry,
            removed:  should_remove,
        }
    }

    pub(crate) fn add_entry(&mut self, from: u64, to: u64) {
        // TODO: add validation
        self.items.push_back(FragmentRegistryEntry {
            path: self.path.join(from.to_string()),
            uid:  from,
            from,
            to,
        });
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FragmentRegistryEntry> {
        self.items.iter()
    }

    pub fn get_by_index(&self, idx: usize) -> Option<&FragmentRegistryEntry> {
        self.items.get(idx)
    }

    pub fn get_by_uid(&self, uid: u64) -> Option<&FragmentRegistryEntry> {
        self.items.iter().find(|e| e.uid == uid)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FragmentRegistryEntry {
    pub path: PathBuf,
    pub uid:  u64,
    pub from: u64,
    pub to:   u64,
}


#[derive(Debug)]
pub struct FragmentRegistryTransaction<'a> {
    registry: &'a mut FragmentRegistry,
    added:    FragmentRegistryEntry,
    removed:  bool,
}

impl<'a> FragmentRegistryTransaction<'a> {
    pub fn commit(self) {
        if self.removed {
            assert!(self.registry.len() > 0);
            _ = self.registry.items.pop_front();
        }

        let entry = self.added.clone();
        self.registry.items.push_back(entry);
    }
}