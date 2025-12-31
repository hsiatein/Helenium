use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HelenyReply {
    pub content: String,
    pub need_help: Option<String>,
}
