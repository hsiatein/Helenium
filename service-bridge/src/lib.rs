use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::BridgeServiceMessage;
use heleny_service::Service;
use tokio::time::Instant;

#[base_service(deps=[])]
pub struct BridgeService {
    endpoint: Endpoint,
}

#[derive(Debug)]
enum WorkerMessage {}

#[async_trait]
impl Service for BridgeService {
    type MessageType = BridgeServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let instance = Self { endpoint };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        name: String,
        role: ServiceRole,
        msg: BridgeServiceMessage,
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

impl BridgeService {}
