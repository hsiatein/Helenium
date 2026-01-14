use anyhow::Context;
use anyhow::Result;
use heleny_proto::AnyMessage;
use heleny_proto::KERNEL_SERVICE;
use heleny_proto::SignedMessage;
use heleny_proto::TokenMessage;
use tokio::sync::mpsc;
use uuid::Uuid;

pub type SubEndpoint = mpsc::Sender<Box<dyn AnyMessage>>;

#[derive(Debug)]
pub struct Endpoint {
    token: Uuid,
    to_bus: mpsc::Sender<TokenMessage>,
    from_bus: Option<mpsc::Receiver<SignedMessage>>,
    to_self: Option<SubEndpoint>,
    from_sub_endpoint: Option<mpsc::Receiver<Box<dyn AnyMessage>>>,
}

impl Endpoint {
    pub fn new(
        token: Uuid,
        to_bus: mpsc::Sender<TokenMessage>,
        from_bus: mpsc::Receiver<SignedMessage>,
        sub_buffer: usize,
    ) -> Self {
        let (to_self, from_sub_endpoint) = mpsc::channel(sub_buffer);
        Self {
            token,
            to_bus,
            from_bus: Some(from_bus),
            to_self: Some(to_self),
            from_sub_endpoint: Some(from_sub_endpoint),
        }
    }

    pub fn new_minimal(token: Uuid, to_bus: mpsc::Sender<TokenMessage>) -> Self {
        Self {
            token,
            to_bus,
            from_bus: None,
            to_self: None,
            from_sub_endpoint: None,
        }
    }

    pub async fn send_box(
        &self,
        target: &str,
        payload: Box<dyn AnyMessage + 'static>,
    ) -> Result<()> {
        let msg = TokenMessage::new(target.to_string(), self.token, payload);
        self.to_bus
            .send(msg)
            .await
            .map_err(|e| anyhow::anyhow!("发送消息到 Kernel 失败: {}", e))
    }

    pub async fn send<T: AnyMessage>(&self, target: &str, payload: T) -> Result<()> {
        self.send_box(target, Box::new(payload)).await
    }

    pub fn create_sub_endpoint(&self) -> Result<SubEndpoint> {
        self.to_self
            .clone()
            .context("最小化启动的 Endpoint 不能使用 SubEndpoint")
    }

    pub fn create_sender_endpoint(&self) -> Endpoint {
        Endpoint::new_minimal(self.token, self.to_bus.clone())
    }

    pub fn send_once(
        &self,
        payload: Box<dyn AnyMessage>,
    ) -> (mpsc::Sender<TokenMessage>, TokenMessage) {
        let msg = TokenMessage::new(KERNEL_SERVICE.to_string(), self.token, payload);
        (self.to_bus.clone(), msg)
    }

    pub fn get_rx(
        &mut self,
    ) -> Result<(
        mpsc::Receiver<SignedMessage>,
        mpsc::Receiver<Box<dyn AnyMessage>>,
    )> {
        let from_bus = self.from_bus.take().context("没有来自 Bus 消息的接收端")?;
        let from_sub_endpoint = self
            .from_sub_endpoint
            .take()
            .context("没有来自 Sub Endpoint 消息的接收端")?;
        Ok((from_bus, from_sub_endpoint))
    }

    pub async fn recv(&mut self) -> Result<SignedMessage> {
        let mut from_bus = self.from_bus.take().context("没有来自 Bus 消息的接收端")?;
        let msg = from_bus.recv().await.context("接收消息失败");
        self.from_bus = Some(from_bus);
        msg
    }
}
