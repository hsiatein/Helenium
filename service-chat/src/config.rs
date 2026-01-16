use heleny_proto::ApiConfig;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleConfig {
    pub api: usize,
    pub preset_path: PathBuf,
    #[serde(default)]
    pub preset: String,
    pub persona_path: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatConfig {
    pub timeout_secs:u64,
    pub api: Vec<ApiConfig>,
    pub heleny: RoleConfig,
    pub planner: RoleConfig,
    pub executor: RoleConfig,
}
