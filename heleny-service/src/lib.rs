use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::AnyMessage;
use heleny_proto::KERNEL_NAME;
use heleny_proto::KERNEL_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ServiceHandle;
use heleny_proto::ServiceRole;
use heleny_proto::SignedMessage;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::Instant;
use tokio::time::Interval;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;
use tracing::Instrument;
use tracing::error;
use tracing::info_span;
use tracing::warn;

mod utils;
pub use utils::*;
mod messages;
pub use messages::*;

/// 服务 trait，定义了服务的基本行为
#[async_trait]
pub trait Service: 'static + HasEndpoint + HasName + Send {
    // 需要实现
    type MessageType: AnyMessage + Send + Sync;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>;
    async fn handle(
        &mut self,
        name: String,
        role: ServiceRole,
        msg: Self::MessageType,
    ) -> Result<()>;
    async fn stop(&mut self);
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()>;
    async fn handle_tick(&mut self, tick: Instant) -> Result<()>;
    async fn handle_resource(&mut self, resource: Resource) -> Result<()>;
    // 默认实现
    fn start(endpoint: Endpoint) -> Result<ServiceHandle> {
        let span = info_span!("", Name = %Self::name());
        let handle = tokio::spawn(
            async move {
                let (sender, fail_msg) = endpoint.send_once(Box::new(
                    KernelServiceMessage::UploadStatus(ServiceSignal::InitFail),
                ));
                let mut service = match Self::new(endpoint).await {
                    Ok(service) => service,
                    Err(e) => {
                        error!("新建服务实例失败, 无法开始: {}", e);
                        let _ = sender.send(fail_msg).await;
                        return Err(anyhow::anyhow!("新建服务实例失败, 无法开始: {}", e));
                    }
                };
                let (from_bus, from_sub_endpoint) = service.endpoint_mut().get_rx()?;
                let mut tick_interval = interval(Duration::from_secs(1));
                tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                service.send_ready().await; // 通知 KernelService 自己初始化完成
                service
                    .launch(from_bus, from_sub_endpoint, tick_interval)
                    .await; // 启动循环
                Ok(())
            }
            .instrument(span),
        );
        Ok(ServiceHandle::new(Self::name().to_string(), handle))
    }
    /// 控制 tokio::select! 的 loop 循环
    async fn launch(
        &mut self,
        mut from_bus: mpsc::Receiver<SignedMessage>,
        mut from_sub_endpoint: mpsc::Receiver<Box<dyn AnyMessage>>,
        mut tick_interval: Interval,
    ) {
        let mut run = true;

        while run {
            tokio::select! {
                Some(msg) = from_bus.recv()=>{
                    self.handle_msg(msg, &mut run).await;
                }
                Some(msg) = from_sub_endpoint.recv()=>{
                    if let Err(e) = self.handle_sub_endpoint(msg).await{
                        warn!("处理 Sub Endpoint 消息错误: {}",e)
                    };
                }
                tick = tick_interval.tick()=>{
                    self.send_alive().await;
                    if let Err(e) = self.handle_tick(tick).await{
                        warn!("处理 Tick 错误: {}",e)
                    };
                }
            }
        }
    }
    /// 处理收到的所有信息
    async fn handle_msg(&mut self, msg: SignedMessage, run: &mut bool) {
        let payload = match Self::downcast(msg.payload) {
            Ok(payload) => payload,
            Err(e) => {
                warn!("收到未知消息类型: {}", e);
                return;
            }
        };
        match payload {
            Ok(message) => {
                if let Err(e) = self.handle(msg.name, msg.role, *message).await {
                    warn!("处理消息时出错: {}", e);
                }
            }
            Err(common_message) => {
                self.handle_common_message(msg.name, msg.role, common_message, run)
                    .await
            }
        };
    }
    /// 处理通用信息
    async fn handle_common_message(
        &mut self,
        _name: String,
        role: ServiceRole,
        message: Box<CommonMessage>,
        run: &mut bool,
    ) {
        match *message {
            CommonMessage::Stop => {
                if role != ServiceRole::System {
                    warn!("非 System 身份不能发送 Stop 消息");
                    return;
                }
                self.stop().await;
                self.send_terminate().await;
                *run = false;
            }
            CommonMessage::Resource(resource) => {
                if let Err(e) = self.handle_resource(resource).await {
                    warn!("处理资源失败: {}", e)
                }
            }
        }
    }
    fn downcast(
        msg: Box<dyn AnyMessage>,
    ) -> Result<Result<Box<Self::MessageType>, Box<CommonMessage>>> {
        // debug!("尝试转换: {:?}", msg);
        let msg = match msg.as_any().downcast::<Self::MessageType>() {
            Ok(msg) => return Ok(Ok(msg)),
            Err(msg) => msg,
        };
        match msg.downcast::<CommonMessage>() {
            Ok(msg) => Ok(Err(msg)),
            Err(_) => Err(anyhow::anyhow!(
                "消息类型转换失败：期望类型为 {} CommonMessage, 但收到的是其他类型",
                std::any::type_name::<Self::MessageType>()
            )),
        }
    }
    async fn send_alive(&self) {
        let _ = self
            .endpoint()
            .send(
                KERNEL_SERVICE,
                KernelServiceMessage::UploadStatus(ServiceSignal::Alive),
            )
            .await;
    }
    async fn send_ready(&self) {
        let _ = self
            .endpoint()
            .send(
                KERNEL_SERVICE,
                KernelServiceMessage::UploadStatus(ServiceSignal::Ready),
            )
            .await;
    }

    async fn send_terminate(&self) {
        let _ = self
            .endpoint()
            .send(
                KERNEL_SERVICE,
                KernelServiceMessage::UploadStatus(ServiceSignal::Terminate("".into())),
            )
            .await;
    }

    async fn get_endpoint_from_kernel(&self, name: &str) -> Result<Endpoint> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .endpoint()
            .send(
                KERNEL_NAME,
                AdminCommand::NewEndpoint {
                    name: name.to_string(),
                    feedback: tx,
                },
            )
            .await;
        rx.await.context("获取 Endpoint 错误")
    }
}

pub trait HasEndpoint: Sync {
    fn endpoint_mut(&mut self) -> &mut Endpoint;
    fn endpoint(&self) -> &Endpoint;
}

pub trait HasName {
    fn name() -> &'static str;
}

pub struct ServiceFactory {
    pub name: &'static str,
    pub deps: &'static [&'static str],
    pub launch: fn(Endpoint) -> Result<ServiceHandle>,
}

pub struct ServiceFactoryVec {
    pub name: &'static str,
    pub deps: Vec<&'static str>,
    pub launch: fn(Endpoint) -> Result<ServiceHandle>,
}

inventory::collect!(ServiceFactory);

pub fn get_factory(name: &str) -> Option<&'static ServiceFactory> {
    inventory::iter::<ServiceFactory>
        .into_iter()
        .find(|&f| f.name == name)
}
