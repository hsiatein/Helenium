use heleny_proto::ToolManual;
use heleny_service::get_from_config_service;
use heleny_service::list_via_fs_service;
use heleny_service::read_via_fs_service;
use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::ToolkitServiceMessage;
use heleny_proto::{message::AnyMessage, role::ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::resource::Resource;
use tracing::info;
use tracing::warn;

use crate::toolkit_config::*;

mod toolkit_config;

#[base_service(deps=["ConfigService"])]
pub struct ToolkitService{
    endpoint:Endpoint,
    tool_manuals:Vec<ToolManual>,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for ToolkitService {
    type MessageType= ToolkitServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let config:ToolkitConfig=get_from_config_service(&endpoint).await?;
        // 读取工具描述
        let tool_paths=list_via_fs_service(&endpoint, config.tools_dir).await?;
        let mut tool_strings=vec![];
        for path in tool_paths {
            let content=read_via_fs_service(&endpoint, path).await?;
            tool_strings.push(content);
        }
        let tool_manuals:Vec<ToolManual>=tool_strings.into_iter().filter_map(|str| {
            match serde_json::from_str(&str) {
                Ok(value)=>Some(value),
                Err(e)=>{
                    warn!("读取 {} 时失败: {:?}",str,e);
                    None
                }
            }
        }).collect();
        info!("读取到 {} 个工具手册",tool_manuals.len());
        // 实例化
        let instance=Self {
            endpoint,
            tool_manuals,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: ToolkitServiceMessage,
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

impl ToolkitService {
    
}
