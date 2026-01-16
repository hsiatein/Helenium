use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    pub short_term_length: i64,
    pub storage_dir: String,
    pub display_length: i64,
}
