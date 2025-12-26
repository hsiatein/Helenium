use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::KernelMessage;
use heleny_proto::message::AnyMessage;
use heleny_proto::name::KERNEL_NAME;
use heleny_proto::resource::Resource;
use heleny_proto::role::ServiceRole;
use heleny_service::UserServiceMessage;
use heleny_service::Service;
use tokio::time::Instant;

use crate::user::User;

mod user;

#[base_service(deps=[])]
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
            UserServiceMessage::Login(frontend_type) => {
                self.users.push(User {
                    name: name.to_string(),
                    frontend_type,
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

impl UserService {}
