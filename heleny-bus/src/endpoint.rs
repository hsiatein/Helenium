use anyhow::Result;
use heleny_proto::{
    kernel_service_message::ServiceSignal,
    kernel_service_message::KernelServiceMessage,
    message::{AnyMessage, SignedMessage, TokenMessage},
    name::KERNEL_SERVICE,
};
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct Endpoint {
    token: Uuid,
    sender: mpsc::Sender<TokenMessage>,
    receiver: mpsc::Receiver<SignedMessage>,
}

impl Endpoint {
    pub fn new(
        token: Uuid,
        sender: mpsc::Sender<TokenMessage>,
        receiver: mpsc::Receiver<SignedMessage>,
    ) -> Self {
        Self {
            token,
            sender,
            receiver,
        }
    }

    pub async fn send(
        &self,
        target: &'static str,
        payload: Box<dyn AnyMessage + 'static>,
    ) -> Result<()> {
        let msg = TokenMessage::new(target, self.token, payload);
        self.sender
            .send(msg)
            .await
            .map_err(|e| anyhow::anyhow!("发送消息到 Kernel 失败: {}", e))
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
        (self.sender.clone(), msg)
    }

    pub async fn recv(&mut self) -> Option<SignedMessage> {
        self.receiver.recv().await
    }
}
