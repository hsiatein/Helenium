pub mod endpoint;
pub mod midware;

use anyhow::Context;
use anyhow::Result;
use heleny_proto::ServiceRole;
use heleny_proto::SignedMessage;
use heleny_proto::TokenMessage;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::Instrument;
use tracing::info_span;
use tracing::warn;
use uuid::Uuid;

use crate::endpoint::Endpoint;

pub enum BusMessage {
    AddEndpoint {
        token: Uuid,
        name: String,
        role: ServiceRole,
        address: mpsc::Sender<SignedMessage>,
        feedback: oneshot::Sender<()>,
    },
    AddProxyEndpoint {
        token: Uuid,
        name: String,
        role: ServiceRole,
        proxy: String,
        feedback: oneshot::Sender<()>,
    },
    RegisterStats {
        sender: mpsc::Sender<(String, String)>,
    },
    SetUser {
        name: String,
    },
}

pub struct BusHandle {
    endpoint_to_bus: mpsc::Sender<TokenMessage>,
    handle_to_bus: mpsc::Sender<BusMessage>,
    handle: JoinHandle<()>,
}

pub struct Bus {
    from_endpoints: Option<mpsc::Receiver<TokenMessage>>,
    from_handle: Option<mpsc::Receiver<BusMessage>>,
    router: HashMap<String, mpsc::Sender<SignedMessage>>,
    tokens: HashMap<Uuid, (String, ServiceRole)>,
    stats_tx: Option<mpsc::Sender<(String, String)>>,
}

impl Bus {
    pub fn new(
        from_endpoints: mpsc::Receiver<TokenMessage>,
        from_handle: mpsc::Receiver<BusMessage>,
        address_map: HashMap<String, mpsc::Sender<SignedMessage>>,
        tokens: HashMap<Uuid, (String, ServiceRole)>,
    ) -> Bus {
        Self {
            from_endpoints: Some(from_endpoints),
            from_handle: Some(from_handle),
            router: address_map,
            tokens,
            stats_tx: None,
        }
    }

    pub fn start(mut bus: Bus) -> JoinHandle<()> {
        let span = info_span!("Bus");
        tokio::spawn(
            async move {
                let mut from_endpoints = match bus.from_endpoints.take() {
                    Some(rx) => rx,
                    None => {
                        warn!("没有接收端 from_endpoints");
                        return;
                    }
                };
                let mut from_handle = match bus.from_handle.take() {
                    Some(rx) => rx,
                    None => {
                        warn!("没有接收端 from_handle");
                        return;
                    }
                };
                loop {
                    tokio::select! {
                        Some(msg) = from_endpoints.recv() => {
                            if let Err(e) = bus.handle_token_message(msg).await{
                                warn!("{}",e);
                            };
                        }
                        Some(msg) = from_handle.recv() => {
                            if let Err(e) = bus.handle_bus_message(msg).await{
                                warn!("{}",e);
                            };
                        }
                    }
                }
            }
            .instrument(span),
        )
    }

    pub async fn handle_bus_message(&mut self, msg: BusMessage) -> Result<()> {
        match msg {
            BusMessage::AddEndpoint {
                token,
                name,
                role,
                address,
                feedback,
            } => {
                self.tokens.insert(token, (name.clone(), role));
                self.router.insert(name, address);
                let _ = feedback.send(());
            }
            BusMessage::RegisterStats { sender } => self.stats_tx = Some(sender),
            BusMessage::SetUser { name } => {
                let user = self
                    .tokens
                    .iter_mut()
                    .find(|(_id, (exist_name, _role))| exist_name == &name)
                    .context("未找到该用户")?;
                user.1.1 = ServiceRole::User;
            }
            BusMessage::AddProxyEndpoint {
                token,
                name,
                role,
                proxy,
                feedback,
            } => {
                self.tokens.insert(token, (name.clone(), role));
                let tx = self
                    .router
                    .get(&proxy)
                    .context(format!("找不到代理 {} 的地址", proxy))?
                    .clone();
                self.router.insert(name, tx);
                let _ = feedback.send(());
            }
        }
        Ok(())
    }

    pub async fn handle_token_message(&mut self, msg: TokenMessage) -> Result<()> {
        // debug!("未签名: {:?}", msg);
        let (name, role) = self
            .tokens
            .get(&msg.token)
            .context("消息携带未知 token, 忽略")?
            .clone();
        let msg = msg.sign(name, role);
        let source = msg.name.clone();
        // if msg.name==heleny_proto::HUB_SERVICE{
        //     tracing::debug!("已签名: 来源 {} 目标{} 内容{:?}", msg.name, msg.target, msg.payload);
        // }
        let target = msg.target.clone();
        self.send(msg)
            .await
            .context(format!("{} 发送给 {} 失败", source, target))?;
        Ok(())
    }

    pub async fn send(&mut self, msg: SignedMessage) -> Result<()> {
        let target = msg.target.clone();
        if let Some(tx) = &self.stats_tx {
            tx.send((msg.name.clone(), msg.target.clone())).await?;
        }
        let tx = self
            .router
            .get(&target)
            .context(format!("未找到服务: {}", target))?;
        tx.send(msg).await?;
        Ok(())
    }
}

impl BusHandle {
    pub fn new(buffer: usize) -> Self {
        let (endpoint_to_bus, from_endpoints) = mpsc::channel(buffer);
        let (handle_to_bus, from_handle) = mpsc::channel(buffer);
        let bus = Bus::new(from_endpoints, from_handle, HashMap::new(), HashMap::new());

        let handle = Bus::start(bus);
        Self {
            endpoint_to_bus,
            handle_to_bus,
            handle,
        }
    }

    pub async fn get_endpoint(
        &mut self,
        name: String,
        buffer: usize,
        role: ServiceRole,
    ) -> Result<Endpoint> {
        let token = Uuid::new_v4();
        let (mpsc_tx, mpsc_rx) = mpsc::channel(buffer);
        let (tx, rx) = oneshot::channel();
        let _ = self
            .handle_to_bus
            .send(BusMessage::AddEndpoint {
                token,
                name,
                role,
                address: mpsc_tx.clone(),
                feedback: tx,
            })
            .await;
        let _ = timeout(Duration::from_secs(5), rx)
            .await
            .context("推送新 Endpoint 信息超时")?
            .context("推送新 Endpoint 信息错误")?;
        Ok(Endpoint::new(
            token,
            self.endpoint_to_bus.clone(),
            mpsc_rx,
            buffer,
        ))
    }

    pub async fn get_proxy_endpoint(
        &mut self,
        name: String,
        proxy: String,
        role: ServiceRole,
    ) -> Result<Endpoint> {
        let token = Uuid::new_v4();
        let (tx, rx) = oneshot::channel();
        let _ = self
            .handle_to_bus
            .send(BusMessage::AddProxyEndpoint {
                token,
                name,
                role,
                proxy,
                feedback: tx,
            })
            .await;
        timeout(Duration::from_secs(5), rx)
            .await
            .context("推送 Proxy Endpoint 信息超时")?
            .context("推送 Proxy Endpoint 信息错误")?;
        Ok(Endpoint::new_minimal(token, self.endpoint_to_bus.clone()))
    }

    pub fn abort(&self) {
        self.handle.abort();
    }

    pub async fn register_stats(&mut self, sender: mpsc::Sender<(String, String)>) -> Result<()> {
        self.handle_to_bus
            .send(BusMessage::RegisterStats { sender })
            .await
            .context("发送统计发送端失败")
    }

    pub async fn set_user(&mut self, name: String) -> Result<()> {
        self.handle_to_bus
            .send(BusMessage::SetUser { name })
            .await
            .context("发送 Set User 失败")
    }
}
