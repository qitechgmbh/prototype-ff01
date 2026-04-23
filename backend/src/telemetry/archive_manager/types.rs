pub type TierRegistry = Vec<(u64, u64, u64)>;

#[derive(Debug, Clone, Default)]
pub struct ArchiveRegistry {
    pub orphans: TierRegistry,
    pub tiers:   Vec<TierRegistry>
}

impl ArchiveRegistry {
    pub fn new(tier_count: usize) -> Self {
        let mut tiers: Vec<TierRegistry> = Vec::new();        

        for _ in 0..tier_count {
            tiers.push(Vec::new());
        }

        Self { 
            orphans: TierRegistry::new(),
            tiers,
        }
    }

    pub fn tier_0(&mut self) -> &TierRegistry {
        self.tiers.get(0).expect("Must exist")
    }

    pub fn tier_0_mut(&mut self) -> &mut TierRegistry {
        self.tiers.get_mut(0).expect("Must exist")
    }
}