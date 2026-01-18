use std::path::PathBuf;
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

impl From<&str> for MemoryContent {
    fn from(value: &str) -> Self {
        MemoryContent::Text(value.to_string())
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
    pub fn temp<T:Into<MemoryContent>>(role: ChatRole, content: T) -> Self {
        Self {
            id:0,
            role,
            time:Local::now(),
            content:content.into(),
        }
    }
}