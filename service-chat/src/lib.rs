use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::ExecutorModel;
use heleny_proto::PlannerModel;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::TASK_SERVICE;
use heleny_service::ChatServiceMessage;
use heleny_service::Service;
use heleny_service::TaskServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::get_tool_descriptions;
use heleny_service::read_via_fs_service;
use tokio::time::Instant;
use tracing::info;

use crate::config::ChatConfig;
use crate::model::HelenyModel;

mod config;
mod model;

pub use heleny_proto::HELENY_SCHEMA;
pub use heleny_proto::PLANNER_SCHEMA;

#[base_service(deps=["ConfigService","FsService","MemoryService","ToolkitService"])]
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
            if let Some(persona_path) = config.heleny.persona_path.take() {
                let persona=read_via_fs_service(&endpoint, persona_path).await?;
                config.heleny.preset=config.heleny.preset.replace("<你可以创建assets/presets/persona.txt文件来进行人物设定，但是不要动这个标签。不创建新文件的话，也可以把这段标签替换成人物设定>", &persona);
            }
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
            endpoint.create_sender_endpoint(),
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
            ChatServiceMessage::Chat { message } => {
                let heleny_reply = self.heleny.chat(message).await?;
                let Some(need_help) = heleny_reply else {
                    return Ok(());
                };
                self.endpoint
                    .send(
                        TASK_SERVICE,
                        TaskServiceMessage::AddTask {
                            task_description: need_help,
                        },
                    )
                    .await
            }
            ChatServiceMessage::GetPlanner { feedback } => {
                let api_config = self
                    .config
                    .api
                    .get(self.config.planner.api)
                    .context("没有此 API 配置")?
                    .to_owned();
                let tool_descriptions = get_tool_descriptions(&self.endpoint).await?;
                let planner = PlannerModel::new(
                    self.config.planner.preset.clone() + &tool_descriptions,
                    api_config,
                );
                let _ = feedback.send(planner);
                Ok(())
            }
            ChatServiceMessage::GetExecutor { feedback } => {
                let api_config = self
                    .config
                    .api
                    .get(self.config.executor.api)
                    .context("没有此 API 配置")?
                    .to_owned();
                let executor = ExecutorModel::new(self.config.executor.preset.clone(), api_config);
                let _ = feedback.send(executor);
                Ok(())
            }
            ChatServiceMessage::TaskFinished { log } => self.heleny.explain_task_result(log).await,
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
