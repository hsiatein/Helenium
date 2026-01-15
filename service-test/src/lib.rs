use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::CanRequestConsent;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::Service;
use heleny_service::TestServiceMessage;
use heleny_service::ToolkitEndpoint;
use tokio::time::Instant;
use uuid::Uuid;

#[base_service(deps=["ToolkitService","UserService"])]
pub struct TestService {
    endpoint: Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for TestService {
    type MessageType = TestServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        // let toolkit_ep=ToolkitEndpoint::new(Uuid::new_v4(), "任务描述".into(), endpoint.create_sender_endpoint());
        // toolkit_ep.request_consent("请求描述".into()).await?;
        // 实例化
        let instance = Self { endpoint };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: TestServiceMessage,
    ) -> Result<()> {
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

impl TestService {}
