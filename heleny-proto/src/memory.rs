use std::path::PathBuf;
use anyhow::Result;
use async_openai::types::chat::ChatCompletionRequestAssistantMessageArgs;
use async_openai::types::chat::ChatCompletionRequestMessage;
use async_openai::types::chat::ChatCompletionRequestSystemMessageArgs;
use async_openai::types::chat::ChatCompletionRequestUserMessageArgs;
use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq,Copy)]
pub enum ChatRole {
    System,
    Assistant,
    User,
}

impl ChatRole {
    pub fn to_str(&self) -> &'static str {
        match self {
            ChatRole::User => "User",
            ChatRole::System => "System",
            ChatRole::Assistant => "Assistant",
        }
    }
    pub fn from(role: &str) -> Self {
        match role {
            "User" => ChatRole::User,
            "System" => ChatRole::System,
            "Assistant" => ChatRole::Assistant,
            _ => panic!("不应出现以外的身份"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MemoryContent {
    Text(String),
    Image(PathBuf),
}

impl From<String> for MemoryContent {
    fn from(value: String) -> Self {
        MemoryContent::Text(value)
    }
}

impl From<PathBuf> for MemoryContent {
    fn from(value: PathBuf) -> Self {
        MemoryContent::Image(value)
    }
}

impl MemoryContent {
    pub fn to_str(&self) -> &str {
        match &self {
            MemoryContent::Text(content) => content,
            MemoryContent::Image(_) => "发送的一张图片.",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: i64,
    pub role: ChatRole,
    pub time: DateTime<Local>,
    pub content: MemoryContent,
}

impl MemoryEntry {
    pub fn new(id: i64, role: ChatRole,time: DateTime<Local>,content: MemoryContent) -> Self {
        Self {
            id,
            role,
            time,
            content,
        }
    }
}

impl TryFrom<&MemoryEntry> for ChatCompletionRequestMessage {
    type Error = anyhow::Error;
    fn try_from(value: &MemoryEntry) -> std::result::Result<Self, Self::Error> {
        let content = value.time.to_string() + ":" + value.content.to_str();
        let msg = match value.role {
            ChatRole::System => ChatCompletionRequestSystemMessageArgs::default()
                .content(content)
                .build()?
                .into(),
            ChatRole::User => ChatCompletionRequestUserMessageArgs::default()
                .content(content)
                .build()?
                .into(),
            ChatRole::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .build()?
                .into(),
        };
        Ok(msg)
    }
}

pub fn build_async_openai_msg(role: ChatRole, content: &str)->Result<ChatCompletionRequestMessage>{
    let time= Local::now();
    let content = time.to_string() + ":" + content;
    let msg = match role {
        ChatRole::System => ChatCompletionRequestSystemMessageArgs::default()
            .content(content)
            .build()?
            .into(),
        ChatRole::User => ChatCompletionRequestUserMessageArgs::default()
            .content(content)
            .build()?
            .into(),
        ChatRole::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
            .content(content)
            .build()?
            .into(),
    };
    Ok(msg)
}