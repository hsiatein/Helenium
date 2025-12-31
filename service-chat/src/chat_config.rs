use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    pub model: String,
    pub api_key_env_var: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleConfig {
    pub api: usize,
    pub preset_path: PathBuf,
    #[serde(default)]
    pub preset: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatConfig {
    pub api: Vec<ApiConfig>,
    pub heleny: RoleConfig,
    pub planner: RoleConfig,
    pub executor: RoleConfig,
}

pub static HELENY_SCHEMA: &'static str = r#"{
  "type": "object",
  "properties": {
    "content": {
      "type": "string",
      "description": "包含动作描述和回复文本的字符串，格式为：（动作）回复内容"
    },
    "need_help": {
      "oneOf": [
        { "type": "null" },
        { 
          "type": "string", 
          "description": "对用户需求的总结。如果需要外部组件（如 Planner 或 Executor）协助，则提供总结；否则为 null。" 
        }
      ]
    }
  },
  "required": ["content", "need_help"],
  "additionalProperties": false
}"#;

pub static PLANNER_SCHEMA: &'static str = r#"{
  "type": "object",
  "properties": {
    "reason": {
      "type": "string",
      "description": "详细解释判断用户需求并选择对应工具的逻辑和原因。"
    },
    "tools": {
      "type": "array",
      "items": {
        "type": "string"
      },
      "description": "需要调用的工具名称列表。如果没有匹配的工具，则返回空数组 []。"
    }
  },
  "required": ["reason", "tools"],
  "additionalProperties": false
}"#;
