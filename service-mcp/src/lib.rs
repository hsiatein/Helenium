use heleny_service::get_from_config_service;
use heleny_service::register_tool_factory;
use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::McpServiceMessage;
use heleny_proto::{AnyMessage, ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::Resource;

use crate::config::Config;
use crate::tool::McpToolFactory;

mod tool;
mod config;

#[base_service(deps=["ConfigService"])]
pub struct McpService{
    endpoint:Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for McpService {
    type MessageType= McpServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let config:Config=get_from_config_service(&endpoint).await?;
        let Config { mcp_servers }=config;
        let factories:Vec<McpToolFactory>=mcp_servers.into_iter().map(|(name,command)|{
            McpToolFactory::new(name, command)
        }).collect();
        for factory in factories {
            register_tool_factory(&endpoint, factory).await;
        }
        // 实例化
        let instance=Self {
            endpoint,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: McpServiceMessage,
    ) -> Result<()>{
        Ok(())
    }
    async fn stop(&mut self){

    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl McpService {
    
}
