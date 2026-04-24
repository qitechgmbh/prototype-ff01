use std::{collections::VecDeque, path::PathBuf};

#[derive(Clone, Debug, Default)]
pub struct FragmentRegistry {
    pub(crate) path:  PathBuf,
    pub(crate) items: VecDeque<FragmentRegistryEntry>,
    capacity: u16,
}

impl FragmentRegistry {
    pub fn new<P: Into<PathBuf>>(path: P, capacity: u16) -> Self {
        assert!(capacity > 0);
        Self {
            path: path.into(),
            items: VecDeque::new(),
            capacity
        }
    }

    pub fn create_append_transaction(
        &mut self,
        from: u64, 
        to:   u64,
    ) -> AppendTransaction<'_> {
        let added = FragmentRegistryEntry {
            path: self.path.join(from.to_string()),
            uid:  from,
            from,
            to,
        };

        let removed = if self.items.len() >= self.capacity as usize {
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