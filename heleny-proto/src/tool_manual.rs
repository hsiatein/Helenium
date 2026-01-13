use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolManual {
    pub name: String,
    pub description: String,
    pub commands: Vec<ToolCommand>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCommand {
    pub name: String,
    pub description: String,
    pub args: Vec<ToolArgument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolArgument {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub arg_type: String,
    pub required: bool,
    pub default: Option<String>,
}

impl ToolManual {
    pub fn get_description(&self) -> ToolDescription {
        ToolDescription {
            name: self.name.clone(),
            description: self.description.clone(),
        }
    }
}
