use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::Endpoint;
use heleny_proto::message::Message;
use heleny_proto::{common_message::CommonMessage, message::AnyMessage};
use tokio::task::JoinHandle;
use tokio::time::{Interval, MissedTickBehavior, interval};
use tracing::{Instrument, error, info_span, warn};

/// 服务句柄，用于管理服务的生命周期
#[derive(Debug)]
pub struct ServiceHandle {
    service_name: &'static str,
    thread_handle: JoinHandle<Result<(), anyhow::Error>>,
}

impl ServiceHandle {
    pub fn new(
        service_name: &'static str,
        thread_handle: JoinHandle<Result<(), anyhow::Error>>,
    ) -> Self {
        Self {
            service_name,
            thread_handle,
        }
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
pub trait Service: 'static + HasEndpoint + HasName + Send {
    type MessageType: AnyMessage + Send + Sync;
    // 需要实现
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>;
    async fn handle(&mut self, msg: Box<Self::MessageType>) -> Result<()>;
    async fn stop(&mut self);
    // 默认实现
    fn start(endpoint: Endpoint) -> Result<ServiceHandle> {
        let span = info_span!("", Name = %Self::name());
        let handle = tokio::spawn(
            async move {
                let (sender, fail_msg) = endpoint.send_init_fail();
                let mut service = match Self::new(endpoint).await {
                    Ok(service) => service,
                    Err(e) => {
                        error!("新建服务实例失败, 无法开始: {}", e);
                        let _ = sender.send(fail_msg).await;
                        return Err(anyhow::anyhow!("新建服务实例失败, 无法开始: {}", e));
                    }
                };
                let mut tick_interval = interval(Duration::from_secs(1));
                tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                service.endpoint().send_alive().await; // 通知 KernelService 自己初始化完成
                service.launch(tick_interval).await; // 启动循环
                Ok(())
            }
            .instrument(span),
        );
        Ok(ServiceHandle::new(Self::name(), handle))
    }
    /// 控制 tokio::select! 的 loop 循环
    async fn launch(&mut self, mut tick_interval: Interval) {
        let mut run = true;
        while run {
            tokio::select! {
                Some(msg) = self.endpoint().recv()=>{
                    self.handle_msg(msg, &mut run).await;
                }
                _ = tick_interval.tick()=>{
                    self.endpoint().send_alive().await;
                }
            }
        }
    }
    /// 处理收到的所有信息
    async fn handle_msg(&mut self, msg: Message, run: &mut bool) {
        let payload = match Self::downcast(msg.payload) {
            Ok(payload) => payload,
            Err(e) => {
                warn!("收到未知消息类型: {}", e);
                return;
            }
        };
        match payload {
            Ok(message) => {
                if let Err(e) = self.handle(message).await {
                    warn!("处理消息时出错: {}", e);
                }
            }
            Err(common_message) => self.handle_common_message(common_message, run).await,
        };
    }
    /// 处理通用信息
    async fn handle_common_message(&mut self, message: Box<CommonMessage>, run: &mut bool) {
        match *message {
            CommonMessage::Stop(oneshot) => {
                self.stop().await;
                let _ = oneshot.send(());
                *run = false;
            }
        }
    }
    fn downcast(
        msg: Box<dyn AnyMessage>,
    ) -> Result<Result<Box<Self::MessageType>, Box<CommonMessage>>> {
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
}

pub trait HasEndpoint {
    fn endpoint(&mut self) -> &mut Endpoint;
}

pub trait HasName {
    fn name() -> &'static str;
}

pub struct ServiceFactory {
    pub name: &'static str,
    pub deps: Vec<&'static str>,
    pub launch: fn(heleny_bus::Endpoint) -> Result<ServiceHandle>,
}

inventory::collect!(ServiceFactory);

pub fn get_factory(name: &'static str) -> Option<&'static ServiceFactory> {
    inventory::iter::<ServiceFactory>
        .into_iter()
        .find(|&f| f.name == name)
}
