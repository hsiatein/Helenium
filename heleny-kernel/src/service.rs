use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::Endpoint;
use heleny_macros::base_service;
use heleny_proto::{common_message::CommonMessage, message::Message, name::KERNEL_NAME};
use heleny_service::{Service, ServiceFactory, ServiceHandle};
use inventory;
use tokio::{sync::oneshot, time::timeout};

use crate::command::AdminCommand;
use heleny_proto::health::HealthStatus;
use heleny_proto::health::KernelHealth;
use tracing::{info, warn};

#[derive(Debug)]
pub enum KernelServiceMessage {
    StopAll,
    Init,
    InitParams(
        Arc<Mutex<KernelHealth>>,
        Arc<Mutex<HashMap<&'static str, ServiceHandle>>>,
    ),
}

#[base_service(deps=[])]
pub struct KernelService {
    endpoint: heleny_bus::Endpoint,
    service_factories: Vec<&'static ServiceFactory>,
    order: Option<Result<Vec<&'static str>>>,
    services: Arc<Mutex<HashMap<&'static str, ServiceHandle>>>,
    health: Arc<Mutex<KernelHealth>>,
}

#[async_trait]
impl Service for KernelService {
    type MessageType = KernelServiceMessage;
    async fn new(mut endpoint: heleny_bus::Endpoint) -> Result<Box<Self>> {
        let service_factories = inventory::iter::<ServiceFactory>.into_iter().collect();
        let (health, services) = match endpoint.recv().await {
            Some(Message {
                target: _,
                token: _,
                payload,
            }) => match Self::downcast(payload) {
                Ok(Ok(health)) => match *health {
                    KernelServiceMessage::InitParams(health, services) => (health, services),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "初始化 KernelService 时未带所需 Arc<Mutex<KernelHealth>>"
                        ));
                    }
                },
                _ => {
                    return Err(anyhow::anyhow!(
                        "初始化 KernelService 时未带所需 Arc<Mutex<KernelHealth>>"
                    ));
                }
            },
            None => {
                return Err(anyhow::anyhow!(
                    "初始化 KernelService 时未带所需 Arc<Mutex<KernelHealth>>"
                ));
            }
        };
        KernelHealth::get_mut(&health)
            .services
            .insert(Self::name(), HealthStatus::Healthy);
        Ok(Box::new(Self {
            endpoint,
            service_factories,
            order: None,
            services,
            health,
        }))
    }
    async fn handle(&mut self, msg: Box<KernelServiceMessage>) -> anyhow::Result<()> {
        match *msg {
            KernelServiceMessage::StopAll => {
                self.stop_all_services().await;
            }
            KernelServiceMessage::Init => {
                self.init_all_services().await;
            }
            KernelServiceMessage::InitParams(_, _) => {}
        }
        Ok(())
    }
    async fn stop(&mut self) {
        info!("{} 已关闭", Self::name())
    }
}

impl KernelService {
    /// 从内核获取 Endpoint
    async fn get_endpoint(&self, name: &'static str) -> Result<Endpoint> {
        let (tx, rx) = oneshot::channel();
        self.send_admin_message(AdminCommand::NewEndpoint(name, tx))
            .await;
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(endpoint)) => Ok(endpoint),
            Ok(Err(e)) => Err(anyhow::anyhow!("获取 Endpoint 时错误: {}", e)),
            Err(e) => Err(anyhow::anyhow!("超时: {}", e)),
        }
    }

    /// 初始化各个服务
    async fn init_all_services(&mut self) {
        info!("开始代理启动各个服务");
        let order = match self.get_order() {
            Ok(order) => order,
            Err(e) => {
                warn!("无法获取Order, 初始化失败: {}", e);
                KernelHealth::get_mut(&self.health).services = KernelHealth::get_mut(&self.health)
                    .services
                    .iter()
                    .map(|(name, _)| (*name, HealthStatus::Stopped))
                    .collect();
                return;
            }
        };
        for name in order {
            match KernelHealth::get_mut(&self.health).services.get(name) {
                None => {
                    warn!("未找到 {}, 忽略", name);
                    continue;
                }
                Some(HealthStatus::Healthy) => {
                    info!("{} 已经是健康状态, 跳过初始化", name);
                    continue;
                }
                _ => info!("开始代理启动 {}", name),
            }
            let factory = match self.service_factories.iter().find(|f| f.name == name) {
                Some(factory) => factory,
                None => {
                    warn!("未找到 {} 的工厂函数, 忽略", name);
                    KernelHealth::get_mut(&self.health)
                        .services
                        .insert(name, HealthStatus::Stopped);
                    continue;
                }
            };
            if !self.is_deps_ready(&factory.deps) {
                warn!("{} 依赖未准备完成, 跳过初始化",name);
                KernelHealth::get_mut(&self.health)
                    .services
                    .insert(name, HealthStatus::Stopped);
                continue;
            }
            let endpoint = match self.get_endpoint(name).await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    warn!("无法获取 Endpoint, {} 启动失败: {}",name, e);
                    KernelHealth::get_mut(&self.health)
                        .services
                        .insert(name, HealthStatus::Stopped);
                    continue;
                }
            };
            let handle = match (factory.launch)(endpoint) {
                Ok(handle) => handle,
                Err(e) => {
                    warn!("初始化 {} 失败: {}",name, e);
                    KernelHealth::get_mut(&self.health)
                        .services
                        .insert(name, HealthStatus::Stopped);
                    continue;
                }
            };
            self.services
                .as_ref()
                .lock()
                .expect("获取 services 锁失败")
                .insert(handle.name(), handle);
            KernelHealth::get_mut(&self.health)
                .services
                .insert(name, HealthStatus::Healthy);
            info!("初始化 {} 成功", name);
        }
        info!("初始化服务完成");
    }

    /// 终止各个服务
    async fn stop_all_services(&mut self) {
        info!("开始代理关闭所有服务");
        let order = match &self.order {
            Some(Ok(order)) => order,
            _ => {
                warn!("无法获取 Order, 直接关闭 Kernel");
                self.send_admin_message(AdminCommand::Shutdown(
                    crate::command::ShutdownStage::StopKernel,
                ))
                .await;
                return;
            }
        };
        for &name in order.iter().rev() {
            info!("开始代理关闭 {}", name);
            {
                let mut health = KernelHealth::get_mut(&self.health);
                match health.services.get(name) {
                    Some(HealthStatus::Healthy) => {
                        health.services.insert(name, HealthStatus::Stopping)
                    }
                    _ => {
                        warn!("{} 非健康状态, 强制杀死", name);
                        match self.services
                        .as_ref()
                        .lock()
                        .expect("获取 services 锁失败")
                        .remove(name){
                        Some(handle) => {
                            handle.abort();
                            info!("强制终止 {} 句柄", name);
                        }
                        None => (),
                        };
                        continue;
                    }
                };
            }
            let (tx, rx) = oneshot::channel();
            let _ = self
                .endpoint
                .send(name, Box::new(CommonMessage::Stop(tx)))
                .await;
            match timeout(Duration::from_secs(5), rx).await {
                Ok(_) => {
                    info!("清理 {} 句柄", name);
                    self.services
                        .as_ref()
                        .lock()
                        .expect("获取 services 锁失败")
                        .remove(name);
                }
                Err(e) => {
                    warn!("获取 {} 关闭反馈失败: {}", name, e);
                    match self.services
                        .as_ref()
                        .lock()
                        .expect("获取 services 锁失败")
                        .remove(name){
                        Some(handle) => {
                            handle.abort();
                            info!("强制终止 {} 句柄", name);
                        }
                        None => continue,
                        };
                }
            }
            KernelHealth::get_mut(&self.health)
                .services
                .insert(name, HealthStatus::Stopped);
        }
        self.send_admin_message(AdminCommand::Shutdown(
            crate::command::ShutdownStage::StopKernel,
        ))
        .await;
    }

    /// 向内核发送管理员消息
    async fn send_admin_message(&self, payload: AdminCommand) {
        let _ = self.endpoint.send(KERNEL_NAME, Box::new(payload)).await;
    }

    /// 获取初始化顺序
    fn get_order(&mut self) -> Result<Vec<&'static str>> {
        if self.order.is_none() {
            let dag_map = self
                .service_factories
                .iter()
                .filter(|f| f.name != Self::name())
                .map(|f| {
                    (
                        f.name,
                        f.deps.iter().copied().collect::<HashSet<&'static str>>(),
                    )
                })
                .collect::<HashMap<&'static str, HashSet<&'static str>>>();
            let order = cal_order(dag_map);
            self.order = Some(order);
        }
        match &self.order {
            None => Err(anyhow::anyhow!("计算初始化顺序失败")),
            Some(Ok(order)) => Ok(order.clone()),
            Some(Err(e)) => Err(anyhow::anyhow!("{}", e)),
        }
    }

    /// 检测前置服务是否准备好
    fn is_deps_ready(&self, deps: &Vec<&'static str>) -> bool {
        deps.iter().all(|&name| {
            KernelHealth::get_mut(&self.health).services.get(name) == Some(&HealthStatus::Healthy)
        })
    }
}

/// 计算依赖顺序
fn cal_order(
    mut dag_map: HashMap<&'static str, HashSet<&'static str>>,
) -> Result<Vec<&'static str>> {
    let mut order = Vec::new();
    let mut last_len = 0;
    while last_len != dag_map.len() {
        last_len = dag_map.len();
        let (new, remain): (
            HashMap<&'static str, HashSet<&'static str>>,
            HashMap<&'static str, HashSet<&'static str>>,
        ) = dag_map.into_iter().partition(|(_, deps)| deps.len() == 0);
        let new = new.keys().copied().collect::<HashSet<&'static str>>();
        dag_map = remain
            .into_iter()
            .map(|(k, deps)| (k, &deps - &new))
            .collect();
        order.extend(new);
    }
    if dag_map.len() == 0 {
        Ok(order)
    } else {
        Err(anyhow::anyhow!("有循环依赖或未知依赖 {:?}", dag_map.keys()))
    }
}

#[cfg(test)]
mod test_order;
