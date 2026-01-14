use anyhow::Result;
use async_trait::async_trait;
use heleny_proto::CanRequestConsent;
use heleny_proto::HelenyProcess;
use heleny_proto::HelenyProcessCommand;
use heleny_proto::HelenyTool;
use heleny_proto::HelenyToolFactory;
use heleny_proto::McpInput;
use heleny_proto::McpOutput;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug)]
pub struct McpToolFactory {
    name: String,
    command: HelenyProcessCommand,
}

impl McpToolFactory {
    pub fn new(name: String, command: HelenyProcessCommand) -> Self {
        Self { name, command }
    }
}

#[async_trait]
impl HelenyToolFactory for McpToolFactory {
    fn name(&self) -> String {
        self.name.clone()
    }
    async fn create(&mut self) -> Result<Box<dyn HelenyTool>> {
        let tool = McpTool::new(self.command.clone());
        Ok(Box::new(tool))
    }
}

#[derive(Debug)]
pub struct McpTool {
    command: HelenyProcessCommand,
    process: Option<HelenyProcess>,
    next_id: u64,
}

impl McpTool {
    pub fn new(command: HelenyProcessCommand) -> Self {
        Self {
            command,
            process: None,
            next_id: 99,
        }
    }

    pub async fn process(&mut self) -> Result<&mut HelenyProcess> {
        if self.process.is_none() {
            let mut process = self.command.spawn().await?;
            let init = json!({
            "jsonrpc":"2.0",
            "id":0,
            "method":"initialize",
            "params":{
                "protocolVersion":"2025-06-18",
                "capabilities":{},
                "clientInfo":{"name":"Heleny","version":"0.1.0"}
            }
            })
            .to_string();
            let initialized =
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string();
            process.write(&init).await?;
            process.read().await?;
            process.write(&initialized).await?;
            self.process = Some(process);
        }
        match &mut self.process {
            Some(process) => Ok(process),
            None => Err(anyhow::anyhow!("获取工具进程失败")),
        }
    }
}

#[async_trait]
impl HelenyTool for McpTool {
    async fn invoke(
        &mut self,
        command: String,
        args: HashMap<String, Value>,
        _request: Box<&dyn CanRequestConsent>,
    ) -> Result<String> {
        let id = self.next_id;
        self.next_id = self.next_id + 1;
        let process = self.process().await?;
        let input = McpInput::from(id, command, args);
        process
            .write(serde_json::to_string(&input)?.as_str())
            .await?;
        loop {
            let output = process.read().await?;
            let Ok(output) = serde_json::from_str::<McpOutput>(&output) else {
                continue;
            };
            return Ok(format!("{:?}", output));
        }
    }
}
