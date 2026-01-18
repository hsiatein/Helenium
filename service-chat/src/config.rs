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
    pub timeout_secs: u64,
    #[serde(default)]
    pub rag_num: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatConfig {
    pub api: Vec<ApiConfig>,
    pub heleny: RoleConfig,
    pub planner: RoleConfig,
    pub executor: RoleConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub model: String,
    pub api_key_env_var: String,
    #[serde(default)]
    pub api_key: String,
}
