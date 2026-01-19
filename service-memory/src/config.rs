use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MemoryConfig {
    pub short_term_length: i64,
    pub display_length: i64,
}
