use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::HUB_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::HubServiceMessage;
use heleny_service::Service;
use tokio::time::Instant;
use tracing::info;

use crate::provider::Provider;

mod provider;

#[base_service(deps=[])]
pub struct HubService {
    endpoint: Endpoint,
    // 资源提供者
    providers: HashMap<String, Provider>,
    // 正在等待资源的服务
    pending: HashMap<String, HashSet<String>>,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for HubService {
    type MessageType = HubServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let instance = Self {
            endpoint,
            providers: HashMap::new(),
            pending: HashMap::new(),
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        name: String,
        _role: ServiceRole,
        msg: HubServiceMessage,
    ) -> Result<()> {
        match msg {
            HubServiceMessage::Publish {
                resource_name,
                receiver,
            } => match self.providers.get_mut(&resource_name) {
                Some(provider) if provider.name != name => Err(anyhow::anyhow!("已经有服务注册了")),
                _ => {
                    let endpoint = self.endpoint.create_sender_endpoint();
                    let subsribers = match self.pending.remove(&resource_name) {
                        Some(subsribers) => subsribers,
                        None => HashSet::new(),
                    };
                    self.providers.insert(
                        resource_name.clone(),
                        Provider::new(name, resource_name.clone(), endpoint, receiver, subsribers)?,
                    );
                    info!("{} 已发布", resource_name);
                    Ok(())
                }
            },
            HubServiceMessage::Unpublish { resource_name } => {
                let provider = self
                    .providers
                    .remove(&resource_name)
                    .context("没有这个资源")?;
                if name == HUB_SERVICE {
                    provider.cancel();
                    info!("取消发布资源 {}", resource_name);
                    Ok(())
                } else {
                    self.providers.insert(resource_name, provider);
                    Err(anyhow::anyhow!("不能取消发布其他服务的资源"))
                }
            }
            HubServiceMessage::Subscribe { resource_name } => {
                match self.providers.get_mut(&resource_name) {
                    Some(provider) => provider.subscribe(name).await,
                    None => {
                        info!("{} 订阅 {}", name, resource_name);
                        self.pending
                            .entry(resource_name)
                            .or_insert(HashSet::new())
                            .insert(name);
                        Ok(())
                    }
                }
            }
            HubServiceMessage::Unsubscribe { resource_name } => {
                match self.providers.get_mut(&resource_name) {
                    Some(provider) => provider.unsubscribe(name).await,
                    None => Ok(()),
                }
            }
            HubServiceMessage::Get {
                resource_name,
                feedback,
            } => {
                let (_, provider) = self
                    .providers
                    .iter()
                    .find(|(r, _)| *r == &resource_name)
                    .context("未找到对应资源")?;
                provider.get(feedback).await
            }
        }
    }
    async fn stop(&mut self) {
        self.providers.iter().for_each(|(_, p)| {
            p.cancel();
        });
    }
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

impl HubService {}
