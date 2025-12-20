use tokio::sync::mpsc;
use heleny_proto::message::{AnyMessage, Message};
use uuid::Uuid;
use std::collections::HashMap;
use anyhow::Result;

pub struct Bus {
    to_kernel: mpsc::Sender<Box<Message>>,
    kernel_recv: mpsc::Receiver<Box<Message>>,
    address_map: HashMap<&'static str, mpsc::Sender<Box<Message>>>,

}

pub struct Endpoint {
    token: Option<Uuid>,
    to_kernel: mpsc::Sender<Box<Message>>,
    service_recv: mpsc::Receiver<Box<Message>>,
}

impl Endpoint {
    pub fn new(token: Option<Uuid>, to_kernel: mpsc::Sender<Box<Message>>, service_recv: mpsc::Receiver<Box<Message>>) -> Self {
        Self { token, to_kernel, service_recv }
    }

    pub async fn send(&self, target: &'static str, payload: Box<dyn AnyMessage + 'static>) -> Result<()> {
        let msg=Box::new(Message::new(target, self.token, payload));
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

    pub fn get_token_endpoint(&mut self, name: &'static str, buffer: usize, token: Uuid) -> Endpoint {
        let (tx,rx)=mpsc::channel(buffer);
        let _=self.address_map.insert(name, tx);
        Endpoint::new(Some(token),self.to_kernel.clone(),rx)
    }

    pub fn get_endpoint(&mut self, name: &'static str, buffer: usize) -> Endpoint {
        let (tx,rx)=mpsc::channel(buffer);
        let _=self.address_map.insert(name, tx);
        Endpoint::new(None,self.to_kernel.clone(),rx)
    }

    pub async fn recv(&mut self) -> Option<Box<Message>> {
        self.kernel_recv.recv().await
    }

    pub async fn send(&self, msg: Box<Message>) -> Result<()> {
        let target=msg.target;
        if let Some(tx) = self.address_map.get::<str>(target) {
            tx.send(msg).await.map_err(|e| anyhow::anyhow!("发送消息到服务 {} 失败: {}", target, e))
        } else {
            Err(anyhow::anyhow!("未找到服务: {}", target))
        }
    }
}