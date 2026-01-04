use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolManual {
    pub name: String,
    pub description: String,
    pub commands: Vec<Command>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub args: Vec<Argument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Argument {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")] // 因为 type 是 Rust 关键字，需要重命名
    pub arg_type: String,
    pub required: bool,
}