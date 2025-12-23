pub mod midware;
pub mod monitor;

use anyhow::Result;
use heleny_proto::{
    common_message::CommonMessage,
    kernel_message::{KernelMessage, ServiceStatus},
    message::{AnyMessage, Message},
    name::KERNEL_NAME, role::ServiceRole,
};
use std::{collections::HashMap, mem::replace};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::midware::Midware;

pub struct Bus {
    endpoint_kernel: mpsc::Sender<Message>,
    kernel_recv: mpsc::Receiver<Message>,
    address_map: HashMap<&'static str, mpsc::Sender<Message>>,
}

#[derive(Debug)]
pub struct Endpoint {
    token: Option<Uuid>,
    endpoint_kernel: mpsc::Sender<Message>,
    service_recv: mpsc::Receiver<Message>,
}

impl Endpoint {
    pub fn new(
        token: Option<Uuid>,
        endpoint_kernel: mpsc::Sender<Message>,
        service_recv: mpsc::Receiver<Message>,
    ) -> Self {
        Self {
            token,
            endpoint_kernel,
            service_recv,
        }
    }

    pub async fn send(
        &self,
        target: &'static str,
        payload: Box<dyn AnyMessage + 'static>,
    ) -> Result<()> {
        let msg = Message::new(target, self.token, payload);
        self.endpoint_kernel
            .send(msg)
            .await
            .map_err(|e| anyhow::anyhow!("发送消息到 Kernel 失败: {}", e))
    }

    pub async fn send_alive(&self) {
        let _ = self.send(KERNEL_NAME, Box::new(KernelMessage::UploadStatus(ServiceStatus::Alive))).await;
    }

    pub async fn send_ready(&self) {
        let _ = self.send(KERNEL_NAME, Box::new(KernelMessage::UploadStatus(ServiceStatus::Ready))).await;
    }

    pub fn send_init_fail(&self) -> (mpsc::Sender<Message>, Message) {
        let msg = Message::new(KERNEL_NAME, self.token, Box::new(KernelMessage::UploadStatus(ServiceStatus::InitFail)));
        (self.endpoint_kernel.clone(), msg)
    }

    pub async fn recv(&mut self) -> Option<Message> {
        self.service_recv.recv().await
    }
}

impl Bus {
    pub fn new(buffer: usize) -> Self {
        let (tx_service, rx) = mpsc::channel(buffer);
        Self {
            endpoint_kernel: tx_service,
            kernel_recv: rx,
            address_map: HashMap::new(),
        }
    }

    pub fn get_token_endpoint(
        &mut self,
        name: &'static str,
        buffer: usize,
        token: Uuid,
    ) -> Endpoint {
        let (tx, rx) = mpsc::channel(buffer);
        let _ = self.address_map.insert(name, tx);
        Endpoint::new(Some(token), self.endpoint_kernel.clone(), rx)
    }

    pub fn get_midware(&mut self, buffer: usize) -> Midware {
        let (tx, rx) = mpsc::channel(buffer);
        let old_rx = replace(&mut self.kernel_recv, rx);
        Midware::new(tx, old_rx)
    }

    pub async fn recv(&mut self) -> Option<Message> {
        self.kernel_recv.recv().await
    }

    pub async fn send_as_kernel(&self, mut msg: Message) -> Result<()> {
        msg.name=Some(KERNEL_NAME);
        msg.role=Some(ServiceRole::System);
        msg.token=None;
        self.send(msg).await
    }

    pub async fn send(&self, msg: Message) -> Result<()> {
        let target = msg.target;
        if let Some(tx) = self.address_map.get::<str>(target) {
            tx.send(msg)
                .await
                .map_err(|e| anyhow::anyhow!("发送消息到服务 {} 失败: {}", target, e))
        } else {
            Err(anyhow::anyhow!("未找到服务: {}", target))
        }
    }

    pub async fn send_common_message(
        &self,
        target: &'static str,
        payload: CommonMessage,
    ) -> Result<()> {
        let msg = Message::new(target, None, Box::new(payload));
        self.send_as_kernel(msg).await
    }
}
