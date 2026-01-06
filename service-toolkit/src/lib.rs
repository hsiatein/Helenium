use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::ToolDescription;
use heleny_proto::ToolManual;
use heleny_service::Service;
use heleny_service::ToolkitServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::list_via_fs_service;
use heleny_service::read_via_fs_service;
use tokio::time::Instant;
use tracing::info;
use tracing::warn;

use crate::toolkit_config::*;

mod toolkit_config;

#[base_service(deps=["ConfigService"])]
pub struct ToolkitService {
    endpoint: Endpoint,
    tool_manuals: Vec<ToolManual>,
    tool_descriptions: Vec<ToolDescription>,
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
        // 实例化
        let instance = Self {
            endpoint,
            tool_manuals,
            tool_descriptions,
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
                let _ = feedback.send(serde_json::to_string(&self.tool_descriptions)?);
            }
            ToolkitServiceMessage::GetManuals {
                tool_names,
                feedback,
            } => {
                let results: Vec<serde_json::Value> = tool_names.into_iter().map(|name| {
                    match self.tool_manuals.iter().find(|tool| tool.name == name) {
                        Some(manual) => {
                            // 将 manual 转换为 json 对象，如果失败则返回错误 json
                            serde_json::to_value(manual).unwrap_or_else(|_| {
                                serde_json::json!({ "error": format!("无法序列化工具手册 {}", name) })
                            })
                        }
                        None => {
                            serde_json::json!({ "error": format!("未找到工具手册 {}", name) })
                        }
                    }
                }).collect();

                if let Ok(json_out) = serde_json::to_string(&results) {
                    let _ = feedback.send(json_out);
                }
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
