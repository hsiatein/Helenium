use heleny_service::{Service};
use async_trait::async_trait;
use heleny_macros::{BaseService};

use crate::command::KernelCommand;

pub enum KernelMessage {
    Shutdown,
    Init(String),
}

#[derive(BaseService)]
pub struct KernelService {
    endpoint: heleny_bus::Endpoint,
}

#[async_trait]
impl Service for KernelService {
    type MessageType = KernelMessage;
    fn new(endpoint:heleny_bus::Endpoint) -> Box<Self> {
        Box::new(Self { endpoint })
    }
    async fn handle(&mut self, msg: Box<KernelMessage>) -> anyhow::Result<()> {
        match msg.as_ref() {
            KernelMessage::Shutdown => {
                println!("KernelService 收到关闭指令，正在关闭...");
                let _=self.endpoint.send("Kernel",Box::new(KernelCommand::Shutdown)).await;
            },
            KernelMessage::Init(service_name) => {
                println!("KernelService 收到初始化服务指令: {}", service_name);
            }
        }
        Ok(())
    }
}