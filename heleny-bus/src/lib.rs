pub mod endpoint;
pub mod midware;

use anyhow::{Context, Result};
use heleny_proto::{
    message::{SignedMessage, TokenMessage},
    role::ServiceRole,
};
use std::{collections::HashMap, time::Duration};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{Instant, MissedTickBehavior, interval, timeout},
};
use tracing::{Instrument, debug, info_span, warn};
use uuid::Uuid;

use crate::endpoint::Endpoint;

pub enum BusMessage {
    AddEndpoint(
        Uuid,
        &'static str,
        ServiceRole,
        mpsc::Sender<SignedMessage>,
        oneshot::Sender<()>,
    ),
}

pub struct BusHandle {
    endpoint_to_bus: mpsc::Sender<TokenMessage>,
    handle_to_bus: mpsc::Sender<BusMessage>,
    handle: JoinHandle<()>,
    stats_rx: Option<mpsc::Receiver<HashMap<&'static str, u64>>>,
}

pub struct Bus {
    from_endpoints: Option<mpsc::Receiver<TokenMessage>>,
    from_handle: Option<mpsc::Receiver<BusMessage>>,
    router: HashMap<&'static str, mpsc::Sender<SignedMessage>>,
    tokens: HashMap<Uuid, (&'static str, ServiceRole)>,
    stats_table: Option<HashMap<&'static str, u64>>,
    stats_tx: mpsc::Sender<HashMap<&'static str, u64>>,
}

impl Bus {
    pub fn new(
        from_endpoints: mpsc::Receiver<TokenMessage>,
        from_handle: mpsc::Receiver<BusMessage>,
        address_map: HashMap<&'static str, mpsc::Sender<SignedMessage>>,
        tokens: HashMap<Uuid, (&'static str, ServiceRole)>,
        stats_tx: mpsc::Sender<HashMap<&'static str, u64>>,
    ) -> Bus {
        Self {
            from_endpoints: Some(from_endpoints),
            from_handle: Some(from_handle),
            router: address_map,
            tokens,
            stats_table: Some(HashMap::new()),
            stats_tx,
        }
    }

    pub fn start(mut bus: Bus) -> JoinHandle<()> {
        let span = info_span!("Bus");
        tokio::spawn(
            async move {
                let mut tick_interval = interval(Duration::from_secs(1));
                tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
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
                        tick = tick_interval.tick() => {
                            if let Err(e) = bus.handle_tick(tick){
                                warn!("{}",e);
                            };
                        }
                    }
                }
            }
            .instrument(span),
        )
    }

    pub fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        let table = self.stats_table.take().context("没有路由统计表")?;
        self.stats_tx.try_send(table)?;
        self.stats_table = Some(HashMap::new());
        Ok(())
    }

    pub async fn handle_bus_message(&mut self, msg: BusMessage) -> Result<()> {
        match msg {
            BusMessage::AddEndpoint(token, name, role, tx, sender) => {
                self.tokens.insert(token, (name, role));
                self.router.insert(name, tx);
                let _ = sender.send(());
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
        debug!("已签名: {:?}", msg);
        let target = msg.target;
        self.send(msg)
            .await
            .context(format!("发送给 {} 失败", target))?;
        Ok(())
    }

    pub async fn send(&self, msg: SignedMessage) -> Result<()> {
        let target = msg.target;
        let tx = self
            .router
            .get(target)
            .context(format!("未找到服务: {}", target))?;
        tx.send(msg).await?;
        Ok(())
    }
}

impl BusHandle {
    pub fn new(buffer: usize) -> Self {
        let (endpoint_to_bus, from_endpoints) = mpsc::channel(buffer);
        let (handle_to_bus, from_handle) = mpsc::channel(buffer);
        let (stats_tx, stats_rx) = mpsc::channel(buffer);
        let bus = Bus::new(
            from_endpoints,
            from_handle,
            HashMap::new(),
            HashMap::new(),
            stats_tx,
        );

        let handle = Bus::start(bus);
        Self {
            endpoint_to_bus,
            handle_to_bus,
            handle,
            stats_rx: Some(stats_rx),
        }
    }

    pub async fn get_endpoint(
        &mut self,
        name: &'static str,
        buffer: usize,
        role: ServiceRole,
    ) -> Result<Endpoint> {
        let token = Uuid::new_v4();
        let (mpsc_tx, mpsc_rx) = mpsc::channel(buffer);
        let (tx, rx) = oneshot::channel();
        let _ = self
            .handle_to_bus
            .send(BusMessage::AddEndpoint(
                token,
                name,
                role,
                mpsc_tx.clone(),
                tx,
            ))
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

    pub fn abort(&self) {
        self.handle.abort();
    }

    pub fn get_stats(&mut self) -> Result<mpsc::Receiver<HashMap<&'static str, u64>>> {
        self.stats_rx.take().context("没有统计接收端")
    }
}
