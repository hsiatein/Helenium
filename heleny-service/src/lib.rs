use async_trait::async_trait;
use tokio::task::JoinHandle;
use anyhow::Result;
use heleny_bus::Endpoint;
use heleny_proto::{common_message::CommonMessage, message::AnyMessage};

/// 服务句柄，用于管理服务的生命周期
pub struct ServiceHandle {
    service_name: &'static str,
    thread_handle: JoinHandle<()>,
}

impl ServiceHandle {
    pub fn new(service_name: &'static str, thread_handle: JoinHandle<()>) -> Self {
        Self { service_name, thread_handle }
    }

    pub fn abort(&self) {
        self.thread_handle.abort();
    }

    pub fn name(&self) -> &'static str {
        self.service_name
    }

}

/// 服务 trait，定义了服务的基本行为
#[async_trait]
pub trait Service: 'static + HasEndpoint + HasName {
    type MessageType: AnyMessage + Send + Sync;
    fn new(endpoint:Endpoint) -> Box<Self>;
    async fn handle(&mut self, msg: Box<Self::MessageType>) -> Result<()>;
    fn dependencies() -> Vec<&'static str> {
        Vec::new()
    }
    async fn start(endpoint:Endpoint) -> Result<ServiceHandle>{
        let mut service=Self::new(endpoint);
        let handle=tokio::spawn(async move {
            while let Some(msg)=service.endpoint().recv().await{
                let payload=Self::downcast(msg.payload);
                let payload=match payload{
                    Ok(payload)=>payload,
                    Err(e)=>{
                        eprintln!("服务 {} 收到未知消息类型: {}", Self::name(), e);
                        continue;
                    }
                };
                if let Err(e)=service.handle(payload).await{
                    eprintln!("服务 {} 处理消息时出错: {}", Self::name(), e);
                }
            }
        });
        Ok(ServiceHandle::new(Self::name(), handle))
    }
    fn stop(&mut self) {

    }
    fn downcast(msg: Box<dyn AnyMessage>) -> Result<Box<Self::MessageType>> {
        msg.as_any().downcast::<Self::MessageType>()
            .map_err(|_| anyhow::anyhow!(
                "消息类型转换失败：期望类型为 {}, 但收到的是其他类型", 
                std::any::type_name::<Self::MessageType>()
            ))
    }

    fn downcast_common(msg: Box<dyn AnyMessage>) -> Result<Box<CommonMessage>> {
        msg.as_any().downcast::<CommonMessage>()
            .map_err(|_| anyhow::anyhow!(
                "消息类型转换失败：期望类型为 CommonMessage, 但收到的是其他类型"
            ))
    }

}

pub trait HasEndpoint {
    fn endpoint(&mut self) -> &mut Endpoint;
}

pub trait HasName {
    fn name() -> &'static str;
}