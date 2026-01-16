use std::any;
use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct HelenyReply {
    pub content: String,
    pub need_help: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredTools {
    pub reason: String,
    pub tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolIntent {
    pub reason: String,
    pub tool: Option<String>,
    pub command: Option<String>,
    #[serde(default)]
    pub args: HashMap<String, Value>,
}

pub fn get_tool_arg<T: DeserializeOwned>(
    args: &mut HashMap<String, Value>,
    name: &str,
) -> Result<T> {
    let Some(arg) = args.remove(name) else {
        return Err(anyhow::anyhow!("没有找到此参数名: {}", name));
    };
    let Ok(arg) = serde_json::from_value(arg.clone()) else {
        return Err(anyhow::anyhow!("{arg} 解析成目标类型 {} 失败",any::type_name::<T>()));
    };
    Ok(arg)
}
