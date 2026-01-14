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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

impl MemoryContent {
    pub fn to_str(&self) -> &str {
        match &self {
            MemoryContent::Text(content) => content,
            MemoryContent::Image(_) => "发送的一张图片.",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayMessage {
    pub id: i64,
    pub role: ChatRole,
    pub time: DateTime<Local>,
    pub content: MemoryContent,
}

impl DisplayMessage {
    pub fn new(id: i64, memory_entry: MemoryEntry) -> Self {
        Self {
            id,
            role: memory_entry.role,
            time: memory_entry.time,
            content: memory_entry.content,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub role: ChatRole,
    pub time: DateTime<Local>,
    pub content: MemoryContent,
}

impl MemoryEntry {
    pub fn new(role: ChatRole, content: MemoryContent) -> Self {
        Self {
            role,
            time: Local::now(),
            content,
        }
    }
    pub fn to_chat_message(&self) -> Result<ChatCompletionRequestMessage> {
        let content = self.time.to_string() + ":" + &self.content.to_str();
        let msg = match self.role {
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
