use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::DISPLAY_MESSAGES;
use heleny_proto::HEALTH;
use heleny_proto::HUB_SERVICE;
use heleny_proto::KERNEL_NAME;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::TOTAL_BUS_TRAFFIC;
use heleny_service::CommonMessage;
use heleny_service::HubServiceMessage;
use heleny_service::KernelMessage;
use heleny_service::Service;
use heleny_service::UserServiceMessage;
use tokio::time::Instant;
use tracing::info;
use tracing::warn;

use crate::user::User;

mod user;

#[base_service(deps=["HubService"])]
pub struct UserService {
    endpoint: Endpoint,
    users: Vec<User>,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for UserService {
    type MessageType = UserServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        endpoint
            .send(
                HUB_SERVICE,
                HubServiceMessage::Subscribe {
                    resource_name: TOTAL_BUS_TRAFFIC.to_string(),
                },
            )
            .await?;
        endpoint
            .send(
                HUB_SERVICE,
                HubServiceMessage::Subscribe {
                    resource_name: DISPLAY_MESSAGES.to_string(),
                },
            )
            .await?;
        endpoint
            .send(
                HUB_SERVICE,
                HubServiceMessage::Subscribe {
                    resource_name: HEALTH.to_string(),
                },
            )
            .await?;
        let instance = Self {
            endpoint,
            users: Vec::new(),
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        name: String,
        _role: ServiceRole,
        msg: UserServiceMessage,
    ) -> Result<()> {
        match msg {
            UserServiceMessage::Login(_frontend_type) => {
                info!("用户 {} 登陆", name);
                self.users.push(User {
                    name: name.to_string(),
                    _frontend_type,
                });
                self.endpoint
                    .send(
                        KERNEL_NAME,
                        KernelMessage::SetUser {
                            name: name.to_string(),
                        },
                    )
                    .await
            }
        }
    }
    async fn stop(&mut self) {
        if let Err(e) = self
            .endpoint
            .send(
                HUB_SERVICE,
                HubServiceMessage::Unsubscribe {
                    resource_name: DISPLAY_MESSAGES.to_string(),
                },
            )
            .await
        {
            warn!("{} 退订 {} 失败: {}", Self::name(), DISPLAY_MESSAGES, e);
        }
        if let Err(e) = self
            .endpoint
            .send(
                HUB_SERVICE,
                HubServiceMessage::Unsubscribe {
                    resource_name: TOTAL_BUS_TRAFFIC.to_string(),
                },
            )
            .await
        {
            warn!("{} 退订 {} 失败: {}", Self::name(), TOTAL_BUS_TRAFFIC, e);
        }
        if let Err(e) = self
            .endpoint
            .send(
                HUB_SERVICE,
                HubServiceMessage::Unsubscribe {
                    resource_name: HEALTH.to_string(),
                },
            )
            .await
        {
            warn!("{} 退订 {} 失败: {}", Self::name(), HEALTH, e);
        }
    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, resource: Resource) -> Result<()> {
        self.send_to_all_users(CommonMessage::Resource(resource))
            .await
    }
}

impl UserService {
    async fn send_to_all_users<T: AnyMessage + Clone>(&self, msg: T) -> Result<()> {
        for user in &self.users {
            if let Err(e) = self.endpoint.send(&user.name, msg.clone()).await {
                warn!("发给所有 User 失败: {}", e)
            };
        }
        Ok(())
    }
}
