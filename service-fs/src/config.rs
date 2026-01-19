use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct FsConfig {
    pub exchange_dir: String,
    pub temp_dir: String,
    pub archive: bool,
    pub archive_path: PathBuf,
}
