use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    pub short_term_length: usize,
    pub storage_dir: String,
    pub display_length: usize,
}
