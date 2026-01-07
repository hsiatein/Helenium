use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::ToolDescription;
use heleny_proto::HelenyToolFactory;
use heleny_proto::ToolManual;
use heleny_service::Service;
use heleny_service::Toolkit;
use heleny_service::ToolkitServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::list_via_fs_service;
use heleny_service::read_via_fs_service;
use tokio::time::Instant;
use tracing::info;
use tracing::warn;

use crate::toolkit_config::*;

mod toolkit_config;

#[base_service(deps=["ConfigService","FsService"])]
pub struct ToolkitService {
    endpoint: Endpoint,
    tool_manuals: HashMap<String,ToolManual>,
    tool_descriptions: Vec<ToolDescription>,
    tool_factories: HashMap<String,Box<dyn HelenyToolFactory>>
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for ToolkitService {
    type MessageType = ToolkitServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config: ToolkitConfig = get_from_config_service(&endpoint).await?;
        // 读取工具描述
        let tool_paths = list_via_fs_service(&endpoint, config.tools_dir).await?;
        let mut tool_strings = Vec::new();
        for path in tool_paths {
            let content = read_via_fs_service(&endpoint, path).await?;
            tool_strings.push(content);
        }
        let tool_manuals: Vec<ToolManual> = tool_strings
            .into_iter()
            .filter_map(|str| match serde_json::from_str(&str) {
                Ok(value) => Some(value),
                Err(e) => {
                    warn!("读取 {} 时失败: {:?}", str, e);
                    None
                }
            })
            .collect();
        let tool_descriptions: Vec<ToolDescription> = tool_manuals
            .iter()
            .map(|manual| manual.get_description())
            .collect();
        info!("读取到 {} 个工具手册", tool_manuals.len());
        let tool_manuals=tool_manuals.into_iter().map(|manual| (manual.name.clone(),manual)).collect();
        // 实例化
        let instance = Self {
            endpoint,
            tool_manuals,
            tool_descriptions,
            tool_factories: HashMap::new(),
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: ToolkitServiceMessage,
    ) -> Result<()> {
        match msg {
            ToolkitServiceMessage::GetIntro { feedback } => {
                let tool_descriptions:Vec<&ToolDescription>=self.tool_descriptions.iter().filter(|des| self.tool_factories.contains_key(&des.name)).collect();
                let _ = feedback.send(serde_json::to_string(&tool_descriptions)?);
            }
            ToolkitServiceMessage::GetToolkit { tool_names, task_id, task_description, feedback }=>{
                let mut manuals=Vec::new();
                let mut tools=HashMap::new();
                for name in &tool_names {
                    let Some(manual)=self.tool_manuals.get(name) else {continue;};
                    let Some(factory)=self.tool_factories.get_mut(name) else {continue;};
                    let Ok(tool)=factory.create().await else {continue;};
                    manuals.push(manual);
                    tools.insert(name.clone(),tool);
                }
                let toolkit=Toolkit::new(task_id, task_description, self.endpoint.create_sender_endpoint(), serde_json::to_string(&manuals).context("序列化工具手册失败")?, tools);
                if let Err(_)=feedback.send(toolkit) {
                    return Err(anyhow::anyhow!("发送工具包失败"))
                };
            }
            ToolkitServiceMessage::Register { factory }=>{
                let name=factory.name();
                info!("成功注册工具: {}",name);
                self.tool_factories.insert(name, factory);
            }
        }
        Ok(())
    }
    async fn stop(&mut self) {}
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl ToolkitService {}
