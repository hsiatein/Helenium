pub mod midware;
pub mod endpoint;

use anyhow::{Context, Result};
use heleny_proto::{
    message::{SignedMessage, TokenMessage},
    role::ServiceRole,
};
use tracing::{Instrument, debug, info_span, warn};
use std::{collections::HashMap, time::Duration};
use tokio::{sync::{mpsc, oneshot}, task::JoinHandle, time::timeout};
use uuid::Uuid;

use crate::endpoint::Endpoint;

pub enum BusMessage {
    AddEndpoint(Uuid,&'static str,ServiceRole,mpsc::Sender<SignedMessage>,oneshot::Sender<()>)
}

pub struct BusHandle {
    endpoint_to_bus: mpsc::Sender<TokenMessage>,
    handle_to_bus: mpsc::Sender<BusMessage>,
    handle: JoinHandle<()>,
}

pub struct Bus {
    from_endpoints: Option<mpsc::Receiver<TokenMessage>>,
    from_handle: Option<mpsc::Receiver<BusMessage>>,
    address_map: HashMap<&'static str, mpsc::Sender<SignedMessage>>,
    tokens: HashMap<Uuid, (&'static str, ServiceRole)>,
}

impl Bus {
    pub fn new(from_endpoints: mpsc::Receiver<TokenMessage>,
    from_handle: mpsc::Receiver<BusMessage>,
    address_map: HashMap<&'static str, mpsc::Sender<SignedMessage>>,
    tokens: HashMap<Uuid, (&'static str, ServiceRole)>)->Bus{
        Self {
            from_endpoints:Some(from_endpoints),from_handle:Some(from_handle),address_map,tokens
        }
    }

    pub fn start(mut bus:Bus)->JoinHandle<()>{
        let span = info_span!("Bus");
        tokio::spawn(async move {
            let mut from_endpoints=match bus.from_endpoints.take() {
                Some(rx)=>rx,
                None => {
                    warn!("没有接收端 from_endpoints");
                    return ;
                }
            };
            let mut from_handle=match bus.from_handle.take() {
                Some(rx)=>rx,
                None => {
                    warn!("没有接收端 from_handle");
                    return ;
                }
            };
            loop {
                tokio::select! {
                    Some(msg) = from_endpoints.recv() => {
                        match bus.handle_token_message(msg).await{
                            Ok(()) => (),
                            Err(e) => {
                                warn!("{}",e);
                            }
                        };
                    }
                    Some(msg) = from_handle.recv() => {
                        match bus.handle_bus_message(msg).await{
                            Ok(()) => (),
                            Err(e) => {
                                warn!("{}",e);
                            }
                        };
                    }
                }
            }
        }.instrument(span))
    }

    pub async fn handle_bus_message(&mut self, msg:BusMessage)->Result<()>{
        match msg {
            BusMessage::AddEndpoint(token,name ,role ,tx,sender)=>{
                self.tokens.insert(token, (name,role));
                self.address_map.insert(name, tx);
                let _=sender.send(());
            }
        }
        Ok(())
    }

    pub async fn handle_token_message(&mut self, msg:TokenMessage)->Result<()>{
        debug!("未签名: {:?}", msg);
        let (name, role) = self.tokens.get(&msg.token).context("消息携带未知 token, 忽略")?.clone();
        let msg=msg.sign(name, role);
        debug!("已签名: {:?}", msg);
        self.send(msg).await?;
        Ok(())
    }

    pub async fn send(&self, msg: SignedMessage) -> Result<()> {
        let target = msg.target;
        let tx=self.address_map.get(target).context(format!("未找到服务: {}", target))?;
        tx.send(msg).await?;
        Ok(())
    }
}

impl BusHandle {
    pub fn new(buffer: usize) -> Self {
        let (endpoint_to_bus, from_endpoints) = mpsc::channel(buffer);
        let (handle_to_bus, from_handle) = mpsc::channel(buffer);
        let bus=Bus::new(from_endpoints, from_handle, HashMap::new(), HashMap::new());
        let handle=Bus::start(bus);
        Self {
            endpoint_to_bus,
            handle_to_bus,
            handle
        }
    }

    pub async fn get_endpoint(
        &mut self,
        name: &'static str,
        buffer: usize,
        role: ServiceRole,
    ) -> Result<Endpoint> {
        let token=Uuid::new_v4();
        let (mpsc_tx, mpsc_rx) = mpsc::channel(buffer);
        let (tx,rx)=oneshot::channel();
        let _=self.handle_to_bus.send(BusMessage::AddEndpoint(token, name, role, mpsc_tx,tx)).await;
        let _=timeout(Duration::from_secs(5), rx).await??;
        Ok(Endpoint::new(token, self.endpoint_to_bus.clone(), mpsc_rx))
    }

    // pub fn get_midware(&mut self, buffer: usize) -> Midware {
    //     let (tx, rx) = mpsc::channel(buffer);
    //     let old_rx = replace(&mut self.from_endpoints, rx);
    //     Midware::new(tx, old_rx)
    // }

    // pub async fn recv(&mut self) -> Result<SignedMessage> {
    //     let msg=self.from_endpoints.recv().await.context("消息接收失败")?;
    //     debug!("未清洗消息: {:?}", msg);
    //     let (name, role) = self.tokens.get(&msg.token).context("消息携带未知 token, 忽略")?.clone();
    //     let msg=msg.sign(name, role);
    //     debug!("已清洗消息: {:?}", msg);
    //     Ok(msg)
    // }

    // pub async fn send_as_kernel(&self, mut msg: SignedMessage) -> Result<()> {
    //     msg.name = KERNEL_NAME;
    //     msg.role = ServiceRole::System;
    //     self.send(msg).await
    // }

    // pub async fn send(&self, msg: SignedMessage) -> Result<()> {
    //     let target = msg.target;
    //     if let Some(tx) = self.address_map.get::<str>(target) {
    //         tx.send(msg)
    //             .await
    //             .map_err(|e| anyhow::anyhow!("发送消息到服务 {} 失败: {}", target, e))
    //     } else {
    //         Err(anyhow::anyhow!("未找到服务: {}", target))
    //     }
    // }

    // pub async fn send_common_message(
    //     &self,
    //     target: &'static str,
    //     payload: CommonMessage,
    // ) -> Result<()> {
    //     let msg = SignedMessage::new(target, KERNEL_NAME,ServiceRole::System, Box::new(payload));
    //     self.send_as_kernel(msg).await
    // }
}
