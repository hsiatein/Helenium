use std::collections::HashMap;

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::ToolArgument;
use crate::ToolCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOutput {
    pub id: u64,
    pub jsonrpc: String,
    pub result: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInput {
    pub id: u64,
    pub jsonrpc: String,
    pub method: String,
    pub params: McpInputParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInputParams {
    pub name: String,
    pub arguments: HashMap<String, Value>,
}

impl McpInput {
    pub fn from(id: u64, command: String, args: HashMap<String, Value>) -> Self {
        let params = McpInputParams {
            name: command,
            arguments: args,
        };
        Self {
            id,
            jsonrpc: "2.0".into(),
            method: "tools/call".into(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolManual {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: McpInputSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInputSchema {
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub properties: HashMap<String, McpArg>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpArg {
    #[serde(rename = "type",default)]
    pub arg_type: String,
    #[serde(default)]
    pub description: String,
    pub default: Option<Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

impl From<McpToolManual> for ToolCommand {
    // fn from(value: McpToolManual) -> Self {
    //     let McpToolManual { name, description, input_schema }=value;
    //     let mut descriptions:Vec<&str>=description.split("Args:").collect();
    //     let description=descriptions.remove(0).trim().to_string();
    //     let arg_descriptions:Vec<&str>=if let Some(arg_descriptions)=descriptions.get(0).copied(){
    //         arg_descriptions.trim().split("\n").collect()
    //     }
    //     else {
    //         Vec::new()
    //     };
    //     let mut arg_descriptions:HashMap<String,String>=arg_descriptions.into_iter().filter_map(|arg_description|{
    //         let mut split:Vec<&str>=arg_description.split(":").collect();
    //         // println!("split: {:?}",split);
    //         let name=split.remove(0).trim().to_string();
    //         let description=split.join(":").trim().to_string();
    //         Some((name,description))
    //     }).collect();
    //     let McpInputSchema { extra:_, required, properties }=input_schema;
    //     let args=properties.into_iter().map(|(arg_name,arg)|{
    //         let McpArg { arg_type, extra:_, default }=arg;
    //         let required=required.contains(&arg_name);
    //         let default=if let Some(default)=default {
    //             Some(default.to_string())
    //         }
    //         else {
    //             None
    //         };
    //         let description=arg_descriptions.remove(&arg_name).unwrap_or(arg_name.clone());
    //         ToolArgument { name:arg_name, description, arg_type, required, default }
    //     }).collect();
    //     Self { name, description, args }
    // }
    fn from(value: McpToolManual) -> Self {
        let McpToolManual {
            name,
            description,
            input_schema,
        } = value;
        let McpInputSchema {
            extra: _,
            required,
            properties,
        } = input_schema;
        let args = properties
            .into_iter()
            .map(|(arg_name, arg)| {
                let McpArg {
                    arg_type,
                    description,
                    default,
                    extra,
                } = arg;
                let required = required.contains(&arg_name);
                let extra_info;
                if extra.is_empty() {
                    extra_info = "".to_string()
                } else {
                    let extra = extra
                        .into_iter()
                        .map(|(k, v)| k + ": " + &v.to_string())
                        .join(", ");
                    extra_info = format!(", 额外信息: {}", extra)
                }
                let description = format!("{}{}", description, extra_info);
                ToolArgument {
                    name: arg_name,
                    description,
                    arg_type,
                    required,
                    default,
                }
            })
            .collect();
        Self {
            name,
            description,
            args,
        }
    }
}
