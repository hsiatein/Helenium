use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::CONFIG_SERVICE;
use heleny_proto::HelenyToolFactory;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::ServiceRole;
use heleny_proto::TOOL_ABSTRACTS;
use heleny_proto::ToolAbstract;
use heleny_proto::ToolDescription;
use heleny_proto::ToolManual;
use heleny_service::ConfigServiceMessage;
use heleny_service::Service;
use heleny_service::Toolkit;
use heleny_service::ToolkitServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::list_via_fs_service;
use heleny_service::publish_resource;
use heleny_service::read_via_fs_service;
use serde_json::Value;
use tokio::sync::watch;
use tokio::time::Instant;
use tracing::info;
use tracing::warn;

use crate::config::*;

mod config;

#[base_service(deps=["ConfigService","FsService","HubService"])]
pub struct ToolkitService {
    endpoint: Endpoint,
    tool_manuals: HashMap<String, ToolManual>,
    tool_descriptions: Vec<ToolDescription>,
    tool_factories: HashMap<String, Box<dyn HelenyToolFactory>>,
    abstract_sender: watch::Sender<ResourcePayload>,
    config: ToolkitConfig,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for ToolkitService {
    type MessageType = ToolkitServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config: ToolkitConfig = get_from_config_service(&endpoint).await?;
        endpoint
            .send(
                CONFIG_SERVICE,
                ConfigServiceMessage::Export {
                    key: "tools_dir".into(),
                    value: Value::String(config.tools_dir.clone()),
                },
            )
            .await?;

        // 发布
        let (abstract_sender, abstract_receiver) = watch::channel(ResourcePayload::ToolAbstracts {
            abstracts: Vec::new(),
        });
        publish_resource(&endpoint, TOOL_ABSTRACTS, abstract_receiver).await?;
        // 实例化
        let mut instance = Self {
            endpoint,
            tool_manuals: HashMap::new(),
            tool_descriptions: Vec::new(),
            tool_factories: HashMap::new(),
            abstract_sender,
            config,
        };
        instance.read_manuals().await?;
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
                let tool_descriptions: Vec<&ToolDescription> = self
                    .tool_descriptions
                    .iter()
                    .filter(|des| self.tool_factories.contains_key(&des.name))
                    .collect();
                let _ = feedback.send(serde_json::to_string(&tool_descriptions)?);
            }
            ToolkitServiceMessage::GetToolkit {
                tool_names,
                task_id,
                task_description,
                feedback,
            } => {
                let mut manuals = Vec::new();
                let mut tools = HashMap::new();
                for name in &tool_names {
                    let Some(manual) = self.tool_manuals.get(name) else {
                        continue;
                    };
                    let Some(factory) = self.tool_factories.get_mut(name) else {
                        continue;
                    };
                    let Ok(tool) = factory.create().await else {
                        continue;
                    };
                    manuals.push(manual);
                    tools.insert(name.clone(), tool);
                }
                let toolkit = Toolkit::new(
                    task_id,
                    task_description,
                    self.endpoint.create_sender_endpoint(),
                    serde_json::to_string(&manuals).context("序列化工具手册失败")?,
                    tools,
                );
                if let Err(_) = feedback.send(toolkit) {
                    return Err(anyhow::anyhow!("发送工具包失败"));
                };
            }
            ToolkitServiceMessage::Register { factory } => {
                let name = factory.name();
                info!("成功注册工具: {}", name);
                self.tool_factories.insert(name, factory);
                self.send_tool_abstracts()?;
            }
            ToolkitServiceMessage::Reload => {
                self.read_manuals().await?;
                info!("工具列表重载完成");
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

impl ToolkitService {
    async fn read_manuals(&mut self) -> Result<()> {
        let tool_paths = list_via_fs_service(&self.endpoint, &self.config.tools_dir).await?;
        let mut tool_strings = Vec::new();
        for path in tool_paths {
            let content = read_via_fs_service(&self.endpoint, path).await?;
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
        self.tool_descriptions = tool_manuals
            .iter()
            .map(|manual| manual.get_description())
            .collect();
        info!("读取到 {} 个工具手册", tool_manuals.len());
        self.tool_manuals = tool_manuals
            .into_iter()
            .map(|manual| (manual.name.clone(), manual))
            .collect();
        self.send_tool_abstracts()
    }
    fn send_tool_abstracts(&self) -> Result<()> {
        let mut abstracts = get_tool_abstracts(&self.tool_manuals);
        abstracts.iter_mut().for_each(|abs| {
            abs.available = self.tool_factories.contains_key(&abs.name);
        });
        self.abstract_sender
            .send(ResourcePayload::ToolAbstracts { abstracts })
            .context("更新 ToolAbstracts 失败")
    }
}

fn get_tool_abstracts(tool_manuals: &HashMap<String, ToolManual>) -> Vec<ToolAbstract> {
    tool_manuals
        .values()
        .map(|manual| manual.clone().into())
        .collect()
}
