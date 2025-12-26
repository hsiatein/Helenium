use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::GuiServiceMessage;
use heleny_proto::message::AnyMessage;
use heleny_proto::resource::Resource;
use heleny_proto::role::ServiceRole;
use heleny_service::Service;
use tokio::time::Instant;

#[base_service(deps=["UserService"])]
pub struct GuiService {
    endpoint: Endpoint,
}

#[derive(Debug)]
enum WorkerMessage {}

#[async_trait]
impl Service for GuiService {
    type MessageType = GuiServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let instance = Self { endpoint };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        name: String,
        role: ServiceRole,
        msg: GuiServiceMessage,
    ) -> Result<()> {
        Ok(())
    }
    async fn stop(&mut self) {}
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }
    async fn handle_tick(&mut self, tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl GuiService {}
