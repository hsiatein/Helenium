use std::{collections::HashMap, fs};

use anyhow::{Context, Result};
use heleny_proto::{HelenyProcessCommand, McpOutput, McpToolManual};
use serde_json::{Value, json};



#[tokio::main]
async fn main()->Result<()>{
    let init = json!({
    "jsonrpc":"2.0",
    "id":0,
    "method":"initialize",
    "params":{
        "protocolVersion":"2025-06-18",
        "capabilities":{},
        "clientInfo":{"name":"MyAgent","version":"0.1.0"}
    }
    }).to_string();
    let initialized = json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string();
    let tools_list = json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}).to_string();

    let str=fs::read_to_string("./script/mcp.json")?;
    let value:Value=serde_json::from_str(&str)?;
    let servers=value.get("mcpServers").context("没有mcpServers")?;
    let servers:HashMap<String,HelenyProcessCommand>=serde_json::from_value(servers.clone())?;
    println!("{:?}",servers);
    let (name,cmd)=servers.into_iter().next().context("获取 iter 失败")?;
    let mut process=cmd.spawn().await?;
    process.write(&init).await?;
    let output=process.read().await?;
    println!("{}",output);
    process.write(&initialized).await?;
    process.write(&tools_list).await?;
    let output=process.read().await?;
    let mut output:McpOutput=serde_json::from_str(&output)?;
    let output:Vec<McpToolManual>=serde_json::from_value(output.result.remove("tools").context("获取tools失败")?)?;
    println!("{:?}",output);
    Ok(())
}