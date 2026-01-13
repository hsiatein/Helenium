use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ToolArgument, ToolCommand};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpOutput {
    pub id: u64,
    pub jsonrpc:String,
    pub result:HashMap<String,Value>
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpInput {
    pub id: u64,
    pub jsonrpc:String,
    pub method:String,
    pub params:McpInputParams
}

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct McpInputParams {
    pub name: String,
    pub arguments:HashMap<String,Value>
}

impl McpInput {
    pub fn from(id:u64,command:String,args:HashMap<String,Value>) -> Self {
        let params=McpInputParams {
            name:command,
            arguments:args
        };
        Self { id, jsonrpc: "2.0".into(), method: "tools/call".into(), params }  
    }
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

impl From<McpToolManual> for ToolCommand {
    fn from(value: McpToolManual) -> Self {
        let McpToolManual { name, description, input_schema }=value;
        let mut descriptions:Vec<&str>=description.split("Args:").collect();
        let description=descriptions.remove(0).trim().to_string();
        let arg_descriptions:Vec<&str>=if let Some(arg_descriptions)=descriptions.get(0).copied(){
            arg_descriptions.trim().split("\n").collect()
        }
        else {
            Vec::new()
        };
        let mut arg_descriptions:HashMap<String,String>=arg_descriptions.into_iter().filter_map(|arg_description|{
            let mut split:Vec<&str>=arg_description.split(":").collect();
            // println!("split: {:?}",split);
            let name=split.remove(0).trim().to_string();
            let description=split.join(":").trim().to_string();
            Some((name,description))
        }).collect();
        let McpInputSchema { r#type:_, title:_, required, properties }=input_schema;
        let args=properties.into_iter().map(|(arg_name,arg)|{
            let McpArg { r#type, title:_, default }=arg;
            let required=required.contains(&arg_name);
            let default=if let Some(default)=default {
                Some(default.to_string())
            }
            else {
                None
            };
            let description=arg_descriptions.remove(&arg_name).unwrap_or(arg_name.clone());
            ToolArgument { name:arg_name, description, arg_type:r#type, required, default }
        }).collect();
        Self { name, description, args }
    }
}