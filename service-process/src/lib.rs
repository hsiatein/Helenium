use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::ProcessServiceMessage;
use heleny_service::Service;
use tokio::time::Instant;

#[cfg(test)]
mod tests;

#[base_service(deps=[])]
pub struct ProcessService {
    endpoint: Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for ProcessService {
    type MessageType = ProcessServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        // 实例化
        let instance = Self { endpoint };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: ProcessServiceMessage,
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

impl ProcessService {}
