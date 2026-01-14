use heleny_service::get_from_config_service;
use heleny_service::read_via_fs_service;
use heleny_service::register_tool_factory;
use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::ToolsServiceMessage;
use heleny_proto::{AnyMessage, ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::Resource;
use tracing::warn;

use crate::comfyui::ComfyuiTool;
use crate::config::ComfyuiConfig;
use crate::config::Config;

mod comfyui;
mod config;

#[base_service(deps=["ConfigService","FsService","ToolkitService"])]
pub struct ToolsService{
    endpoint:Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for ToolsService {
    type MessageType= ToolsServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let config:Config=get_from_config_service(&endpoint).await?;
        let Config {comfyui_config} =config;
        // 初始化comfyui
        match init_comfyui(&endpoint,comfyui_config).await {
            Ok(factory)=>{
                register_tool_factory(&endpoint, factory).await;
            }
            Err(e)=> {
                warn!("初始化 ComfyUI 工具失败: {e}");
            }
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
        _msg: ToolsServiceMessage,
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

impl ToolsService {
    
}

async fn init_comfyui(endpoint:&Endpoint,config:ComfyuiConfig)->Result<ComfyuiTool>{
    let ComfyuiConfig { api_key, base_url, base_prompt_path }=config;
    let base_prompt=read_via_fs_service(&endpoint, base_prompt_path).await?;
    let tool=ComfyuiTool::new(base_url, base_prompt, api_key).await?;
    Ok(tool)
}