use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::WebuiServiceMessage;
use heleny_proto::{message::AnyMessage, role::ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::resource::Resource;


#[base_service(deps=[])]
pub struct WebuiService{
    endpoint:Endpoint,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for WebuiService {
    type MessageType= WebuiServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let instance=Self {
            endpoint,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: WebuiServiceMessage,
    ) -> Result<()>{
        Ok(())
    }
    async fn stop(&mut self){

    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl WebuiService {
    
}
