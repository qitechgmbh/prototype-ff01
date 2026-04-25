#[derive(Debug, Clone)]
pub struct Archive {
    pub from: u64,
    pub to:   u64,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub range: u64,
    pub to:   u64,
}