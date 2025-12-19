use crate::service::{Service};
use async_trait::async_trait;
use heleny_macros::{HasEndpoint, HasName};

pub enum KernelMessage {
    Shutdown,
    Init(String),
}

#[derive(HasEndpoint, HasName)]
pub struct KernelService {
    endpoint: crate::bus::Endpoint,
}

#[async_trait]
impl Service for KernelService {
    type MessageType = KernelMessage;
    fn new(endpoint:crate::bus::Endpoint) -> Box<Self> {
        Box::new(Self { endpoint })
    }
    async fn handle(&mut self, msg: Box<KernelMessage>) -> anyhow::Result<()> {
        match msg.as_ref() {
            KernelMessage::Shutdown => {
                println!("KernelService 收到关闭指令，正在关闭...");
                // 在这里添加关闭逻辑
            },
            KernelMessage::Init(service_name) => {
                println!("KernelService 收到初始化服务指令: {}", service_name);
                // 在这里添加初始化服务逻辑
            }
        }
        Ok(())
    }
}