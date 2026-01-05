use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HelenyReply {
    pub content: String,
    pub need_help: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequiredTools {
    pub reason: String,
    pub tools: Option<Vec<String>>
}