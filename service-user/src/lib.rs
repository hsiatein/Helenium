use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::ConsentRequestion;
use heleny_proto::DISPLAY_MESSAGES;
use heleny_proto::HEALTH;
use heleny_proto::KERNEL_NAME;
use heleny_proto::Resource;
use heleny_proto::SCHEDULE;
use heleny_proto::ServiceRole;
use heleny_proto::TASK_ABSTRACT;
use heleny_proto::TOOL_ABSTRACTS;
use heleny_proto::TOTAL_BUS_TRAFFIC;
use heleny_proto::UserDecision;
use heleny_service::CommonMessage;
use heleny_service::KernelMessage;
use heleny_service::Service;
use heleny_service::UserServiceMessage;
use heleny_service::WebuiServiceMessage;
use heleny_service::subscribe_resource;
use heleny_service::unsubscribe_resource;
use tokio::time::Instant;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

use crate::user::User;

mod user;

static RESOURCES: [&'static str; 6] = [
    DISPLAY_MESSAGES,
    TOTAL_BUS_TRAFFIC,
    HEALTH,
    TASK_ABSTRACT,
    SCHEDULE,
    TOOL_ABSTRACTS,
];

#[base_service(deps=["HubService"])]
pub struct UserService {
    endpoint: Endpoint,
    users: Vec<User>,
    consent_requestions: HashMap<Uuid, ConsentRequestion>,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for UserService {
    type MessageType = UserServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        for resource in RESOURCES {
            subscribe_resource(&endpoint, resource).await?;
        }
        let instance = Self {
            endpoint,
            users: Vec::new(),
            consent_requestions: HashMap::new(),
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
            UserServiceMessage::RequestConsent { body } => {
                let request_id = Uuid::new_v4();
                let requestion = body.to_frontend(request_id);
                self.consent_requestions.insert(request_id, body);
                info!("收到新请求");
                self.send_to_all_users(WebuiServiceMessage::UserDecision(
                    UserDecision::ConsentRequestions(vec![requestion]),
                ))
                .await
            }
            UserServiceMessage::ListConsentRequestions { feedback } => {
                let reqs = self
                    .consent_requestions
                    .iter()
                    .map(|(k, v)| v.to_frontend(*k))
                    .collect();
                let _ = feedback.send(reqs);
                Ok(())
            }
            UserServiceMessage::Logout => {
                self.users.retain(|user| user.name != name);
                Ok(())
            }
            UserServiceMessage::MakeDecision { req_id, approval } => {
                let cr = self
                    .consent_requestions
                    .remove(&req_id)
                    .context("未找到此请求")?;
                if approval {
                    info!("用户同意了 {:?}", cr);
                } else {
                    info!("用户拒绝了 {:?}", cr);
                }
                let _ = cr.feedback.send(approval);
                Ok(())
            }
        }
    }
    async fn stop(&mut self) {
        for resource in RESOURCES {
            if let Err(e) = unsubscribe_resource(&self.endpoint, resource).await {
                warn!("{} 退订 {} 失败: {}", Self::name(), resource, e);
            }
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
