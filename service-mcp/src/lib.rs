use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::McpServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use heleny_service::register_tool_factory;
use heleny_service::update_config_service;
use tokio::time::Instant;
use tracing::info;

use crate::config::Config;
use crate::tool::McpToolFactory;

mod config;
mod tool;

#[base_service(deps=["ConfigService"])]
pub struct McpService {
    endpoint: Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for McpService {
    type MessageType = McpServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        // 实例化
        let instance = Self { endpoint };
        info!("载入 MCP 工具");
        instance.load().await?;
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: McpServiceMessage,
    ) -> Result<()> {
        match msg {
            McpServiceMessage::Reload => {
                info!("重载 MCP 工具");
                self.load().await.context("重载 MCP 工具失败")?;
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

impl McpService {
    async fn load(&self)->Result<()> {
        update_config_service(&self.endpoint).await.context("重载失败: 更新 config 失败")?;
        let config: Config = get_from_config_service(&self.endpoint).await.context("重载失败: 获取 config 失败")?;
        let factories: Vec<McpToolFactory> = config
            .mcp_servers
            .into_iter()
            .map(|(name, command)| McpToolFactory::new(name, command))
            .collect();
        for factory in factories {
            register_tool_factory(&self.endpoint, factory).await;
        }
        Ok(())
    }
}
