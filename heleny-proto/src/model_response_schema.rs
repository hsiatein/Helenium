use std::str::FromStr;

use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

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
    pub args: Vec<ToolArg>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolArg {
    pub name: String,
    pub value: String,
}

pub fn get_tool_arg<T: FromStr>(args: &Vec<ToolArg>, name: &str) -> Result<T> {
    let Some(arg) = args.iter().find(|arg| arg.name == name) else {
        return Err(anyhow::anyhow!("没有找到此参数名: {}", name));
    };
    let Ok(arg) = arg.value.parse::<T>() else {
        return Err(anyhow::anyhow!("解析成目标类型失败"));
    };
    Ok(arg)
}
