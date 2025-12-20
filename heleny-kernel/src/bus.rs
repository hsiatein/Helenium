use tokio::sync::mpsc;
use crate::service::Message;
use std::collections::HashMap;
use anyhow::Result;

pub struct Bus {
    to_kernel: mpsc::Sender<Box<Message>>,
    kernel_recv: mpsc::Receiver<Box<Message>>,
    address_map: HashMap<&'static str, mpsc::Sender<Box<Message>>>,
}

pub struct Endpoint {
    to_kernel: mpsc::Sender<Box<Message>>,
    service_recv: mpsc::Receiver<Box<Message>>,
}

impl Endpoint {
    pub fn new(to_kernel: mpsc::Sender<Box<Message>>, service_recv: mpsc::Receiver<Box<Message>>) -> Self {
        Self { to_kernel, service_recv }
    }

    pub async fn send(&self, msg: Box<Message>) -> Result<()> {
        self.to_kernel.send(msg).await.map_err(|e| anyhow::anyhow!("发送消息到 Kernel 失败: {}", e))
    }

    pub async fn recv(&mut self) -> Option<Box<Message>> {
        self.service_recv.recv().await
    }
}

impl Bus {
    pub fn new(buffer: usize) -> Self {
        let (tx_service, rx) = mpsc::channel(buffer);
        Self { to_kernel: tx_service, kernel_recv: rx, address_map: HashMap::new() }
    }

    pub fn get_endpoint(&mut self, name: &'static str, buffer: usize) -> Endpoint {
        let (tx,rx)=mpsc::channel(buffer);
        let _=self.address_map.insert(name, tx);
        Endpoint::new(self.to_kernel.clone(),rx)
    }

    pub async fn recv(&mut self) -> Option<Box<Message>> {
        self.kernel_recv.recv().await
    }

    pub async fn send(&self, msg: Box<Message>) -> Result<()> {
        let service_name = msg.target.clone();
        if let Some(tx) = self.address_map.get::<str>(&service_name) {
            tx.send(msg).await.map_err(|e| anyhow::anyhow!("发送消息到服务 {} 失败: {}", service_name, e))
        } else {
            Err(anyhow::anyhow!("未找到服务: {}", service_name))
        }
    }
}