use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::message::AnyMessage;
use heleny_proto::resource::Resource;
use heleny_proto::role::ServiceRole;
use heleny_service::ChatServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use heleny_service::read_via_fs_service;
use tokio::time::Instant;
use tracing::info;

use crate::chat_config::ChatConfig;
use crate::model::HelenyModel;

mod chat_config;
mod heleny_reply;
mod model;

pub use chat_config::HELENY_SCHEMA;
pub use chat_config::PLANNER_SCHEMA;

#[base_service(deps=["ConfigService","FsService","MemoryService"])]
pub struct ChatService {
    endpoint: Endpoint,
    config: ChatConfig,
    heleny: HelenyModel,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for ChatService {
    type MessageType = ChatServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let mut config: ChatConfig = get_from_config_service(&endpoint).await?;
        // 读取 API KEY
        for api in &mut config.api {
            if api.api_key.is_empty() {
                api.api_key =
                    std::env::var(&api.api_key_env_var).context("读取 API KEY 环境变量失败")?;
            }
        }
        // 读取预设
        if config.heleny.preset.is_empty() {
            config.heleny.preset =
                read_via_fs_service(&endpoint, &config.heleny.preset_path).await?;
        }
        if config.planner.preset.is_empty() {
            config.planner.preset =
                read_via_fs_service(&endpoint, &config.planner.preset_path).await?;
        }
        if config.executor.preset.is_empty() {
            config.executor.preset =
                read_via_fs_service(&endpoint, &config.executor.preset_path).await?;
        }
        info!("Heleny 预设读取完成");
        // 构造 Heleny
        let heleny = HelenyModel::new(
            config.heleny.preset.clone(),
            config
                .api
                .get(config.heleny.api)
                .context("没有此 API 配置")?
                .to_owned(),
            endpoint.create_sender_endpoint()
        );
        // 构造实例
        let instance = Self {
            endpoint,
            config,
            heleny,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: ChatServiceMessage,
    ) -> Result<()> {
        match msg {
            ChatServiceMessage::Chat { message }=>{
                let heleny_reply=self.heleny.chat(message).await?;
                match heleny_reply {
                    Some(_need_help)=>{
                        Ok(())
                    }
                    None=>{
                        Ok(())
                    }
                }
            }
        }
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

impl ChatService {}

#[cfg(test)]
mod tests;
