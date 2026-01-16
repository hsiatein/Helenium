use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub comfyui_config:ComfyuiConfig
}

#[derive(Deserialize, Debug)]
pub struct ComfyuiConfig {
    pub api_key_env_var:String,
    pub base_url_env_var:String,
    pub base_auth_env_var:String,
    pub base_prompt_path:String,
}