use anyhow::{Context, Result};
use heleny_proto::{
    kernel_service_message::KernelServiceMessage,
    kernel_service_message::ServiceSignal,
    message::{AnyMessage, SignedMessage, TokenMessage},
    name::KERNEL_SERVICE,
};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct Endpoint {
    token: Uuid,
    to_bus: mpsc::Sender<TokenMessage>,
    from_bus: Option<mpsc::Receiver<SignedMessage>>,
    to_self: mpsc::Sender<Box<dyn AnyMessage>>,
    from_sub_endpoint: Option<mpsc::Receiver<Box<dyn AnyMessage>>>,
}

impl Endpoint {
    pub fn new(
        token: Uuid,
        to_bus: mpsc::Sender<TokenMessage>,
        from_bus: mpsc::Receiver<SignedMessage>,
        sub_buffer: usize,
    ) -> Self {
        let (to_self,from_sub_endpoint)=mpsc::channel(sub_buffer);
        Self {
            token,
            to_bus,
            from_bus:Some(from_bus),
            to_self,
            from_sub_endpoint:Some(from_sub_endpoint),
        }
    }

    pub async fn send(
        &self,
        target: &'static str,
        payload: Box<dyn AnyMessage + 'static>,
    ) -> Result<()> {
        let msg = TokenMessage::new(target, self.token, payload);
        self.to_bus
            .send(msg)
            .await
            .map_err(|e| anyhow::anyhow!("发送消息到 Kernel 失败: {}", e))
    }

    pub fn get_sub_endpoint(
        &self,
    ) -> mpsc::Sender<Box<dyn AnyMessage>> {
        self.to_self.clone()
    }

    pub async fn send_alive(&self) {
        let _ = self
            .send(
                KERNEL_SERVICE,
                Box::new(KernelServiceMessage::UploadStatus(ServiceSignal::Alive)),
            )
            .await;
    }

    pub async fn send_ready(&self) {
        let _ = self
            .send(
                KERNEL_SERVICE,
                Box::new(KernelServiceMessage::UploadStatus(ServiceSignal::Ready)),
            )
            .await;
    }

    pub async fn send_terminate(&self) {
        let _ = self
            .send(
                KERNEL_SERVICE,
                Box::new(KernelServiceMessage::UploadStatus(ServiceSignal::Terminate)),
            )
            .await;
    }

    pub fn send_init_fail(&self) -> (mpsc::Sender<TokenMessage>, TokenMessage) {
        let msg = TokenMessage::new(
            KERNEL_SERVICE,
            self.token,
            Box::new(KernelServiceMessage::UploadStatus(ServiceSignal::InitFail)),
        );
        (self.to_bus.clone(), msg)
    }

    pub fn get_rx(&mut self) -> Result<(mpsc::Receiver<SignedMessage>,mpsc::Receiver<Box<dyn AnyMessage>>)> {
        let from_bus=self.from_bus.take().context("没有来自 Bus 消息的接收端")?;
        let from_sub_endpoint=self.from_sub_endpoint.take().context("没有来自 Sub Endpoint 消息的接收端")?;
        Ok((from_bus,from_sub_endpoint))
    }

    pub async fn recv(&mut self) -> Result<SignedMessage> {
        let mut from_bus=self.from_bus.take().context("没有来自 Bus 消息的接收端")?;
        let msg=from_bus.recv().await.context("接收消息失败");
        self.from_bus=Some(from_bus);
        msg
    }
}
