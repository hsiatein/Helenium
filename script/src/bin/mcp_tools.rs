use std::collections::HashMap;
use std::fs;

use anyhow::Context;
use anyhow::Result;
use heleny_proto::HelenyProcessCommand;
use heleny_proto::McpOutput;
use heleny_proto::McpToolManual;
use heleny_proto::ToolManual;
use serde_json::Value;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let init = json!({
    "jsonrpc":"2.0",
    "id":0,
    "method":"initialize",
    "params":{
        "protocolVersion":"2025-06-18",
        "capabilities":{},
        "clientInfo":{"name":"MyAgent","version":"0.1.0"}
    }
    })
    .to_string();
    let initialized = json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string();
    let tools_list = json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}).to_string();

    let str = fs::read_to_string("./script/mcp.json")?;
    let value: Value = serde_json::from_str(&str)?;
    let servers = value.get("mcpServers").context("没有mcpServers")?;
    let servers: HashMap<String, HelenyProcessCommand> = serde_json::from_value(servers.clone())?;
    println!("{:?}", servers);
    let (name, cmd) = servers.into_iter().next().context("获取 iter 失败")?;
    let mut process = cmd.spawn().await?;
    process.write(&init).await?;
    let output = process.read().await?;
    println!("{}", output);
    process.write(&initialized).await?;
    process.write(&tools_list).await?;
    let output = process.read().await?;
    println!("{}", output);
    let mut output: McpOutput = serde_json::from_str(&output)?;
    let output: Vec<McpToolManual> =
        serde_json::from_value(output.result.remove("tools").context("获取tools失败")?)?;
    println!("{:?}", output);
    let path = format!("./script/{}.json", name);
    let commands: Vec<heleny_proto::ToolCommand> =
        output.into_iter().map(|tool| tool.into()).collect();
    let description = commands
        .iter()
        .map(|command| command.name.as_str())
        .collect::<Vec<&str>>()
        .join(", ")+".";
    let manual = ToolManual {
        name,
        description,
        commands,
    };
    fs::write(path, serde_json::to_string_pretty(&manual)?)?;
    Ok(())
}
