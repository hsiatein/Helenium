use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub model: String,
    pub api_key_env_var: String,
    #[serde(default)]
    pub api_key: String,
}
