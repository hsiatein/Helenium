use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolManual {
    pub name: String,
    pub description: String,
    pub commands: Vec<ToolCommand>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCommand {
    pub name: String,
    pub description: String,
    pub args: Vec<ToolArgument>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolArgument {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    pub required: bool,
    pub default: Option<Value>,
}

impl ToolManual {
    pub fn get_description(&self) -> ToolDescription {
        ToolDescription {
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAbstract {
    pub name: String,
    pub description: String,
    pub commands: HashMap<String, String>,
    pub available: bool,
    pub enable: bool,
}

impl From<ToolManual> for ToolAbstract {
    fn from(value: ToolManual) -> Self {
        let ToolManual {
            name,
            description,
            commands,
        } = value;
        let commands: HashMap<String, String> = commands
            .into_iter()
            .map(|cmd| {
                let ToolCommand {
                    name,
                    description,
                    args: _,
                } = cmd;
                (name, description)
            })
            .collect();
        Self {
            name,
            description,
            commands,
            available: false,
            enable: true,
        }
    }
}
