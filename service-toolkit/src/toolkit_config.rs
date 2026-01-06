use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ToolkitConfig {
    pub tools_dir: String,
}
