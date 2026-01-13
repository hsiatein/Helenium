use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpOutput {
    pub id: u64,
    pub jsonrpc:String,
    pub result:HashMap<String,Value>
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpToolManual {
    pub name:String,
    pub description:String,
    #[serde(rename = "inputSchema")]
    pub input_schema: McpInputSchema,
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpInputSchema {
    pub r#type:String,
    pub title:String,
    pub required:Vec<String>,
    pub properties: HashMap<String,McpArg>
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpArg {
    pub r#type:String,
    pub title:String,
    pub default:Option<Value>
}