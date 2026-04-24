use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct FragmentRegistry {
    pub(crate) items:    VecDeque<FragmentRegistryEntry>,
    pub(crate) capacity: usize ,
}

impl FragmentRegistry {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            items: VecDeque::new(),
            capacity
        }
    }

    pub fn start_append(&mut self, entry: FragmentRegistryEntry) -> AppendTransaction<'_> {
        let added = entry;

        let removed = if self.items.len() >= self.capacity {
            self.items.front().cloned()
        } else {
            None
        };

        AppendTransaction {
            registry: self,
            added,
            removed,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &FragmentRegistryEntry> {
        self.items.iter()
    }
}

#[derive(Debug, Clone, Default)]
pub struct FragmentRegistryEntry {
    pub uuid:  u128,
    pub from: u64,
    pub to:   u64,
}

#[derive(Debug)]
pub struct AppendTransaction<'a> {
    registry: &'a mut FragmentRegistry,
    added:    FragmentRegistryEntry,
    removed:  Option<FragmentRegistryEntry>,
}

impl<'a> AppendTransaction<'a> {
    pub fn added_entry(&self) -> &FragmentRegistryEntry {
        &self.added
    }

    pub fn removed_entry(&self) -> &Option<FragmentRegistryEntry> {
        &self.removed
    }

    pub fn commit(self) {
        if self.removed.is_some() {
            assert!(self.registry.len() > 0);
            _ = self.registry.items.pop_front();
        }

        let entry = self.added.clone();
        self.registry.items.push_back(entry);
    }
}