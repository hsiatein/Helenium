use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ToolIntent {
    pub reason: String,
    pub tool: Option<String>,
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<ToolArg>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolArg {
    pub name: String,
    pub value: String,
}
