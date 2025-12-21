use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};

use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::Endpoint;
use heleny_macros::base_service;
use heleny_proto::{common_message::CommonMessage, name::KERNEL_NAME};
use heleny_service::{HasName, Service, ServiceFactory};
use inventory;
use tokio::{sync::oneshot, time::timeout};

use crate::{command::AdminCommand, health::new_kernel_health};
use heleny_proto::health::HealthStatus;
use heleny_proto::health::KernelHealth;

pub enum KernelServiceMessage {
    StopAll,
    Init,
    GetHealth(oneshot::Sender<KernelHealth>),
}

#[base_service(deps=[])]
pub struct KernelService {
    endpoint: heleny_bus::Endpoint,
    service_factories: Vec<&'static ServiceFactory>,
    order: Option<Result<Vec<&'static str>>>,
    health: KernelHealth,
}

#[async_trait]
impl Service for KernelService {
    type MessageType = KernelServiceMessage;
    fn new(endpoint: heleny_bus::Endpoint) -> Result<Box<Self>> {
        let service_factories = inventory::iter::<ServiceFactory>
            .into_iter()
            .filter(|factory| factory.name != KernelService::name())
            .collect();
        Ok(Box::new(Self {
            endpoint,
            service_factories,
            order: None,
            health: new_kernel_health(),
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
            KernelServiceMessage::GetHealth(sender) => {
                let _ = sender.send(self.health.clone());
            }
        }
        Ok(())
    }
    async fn stop(&mut self) {
        println!("{} 已关闭", Self::name())
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
        let order = match self.get_order() {
            Ok(order) => order,
            Err(e) => {
                eprintln!("无法获取Order, 初始化失败: {}", e);
                self.health.services = self
                    .health
                    .services
                    .iter()
                    .map(|(name, _)| (*name, HealthStatus::Stopped))
                    .collect();
                return;
            }
        };
        self.health.kernel = HealthStatus::Healthy;
        for name in order {
            let factory = match self.service_factories.iter().find(|f| f.name == name) {
                Some(factory) => factory,
                None => {
                    self.health.services.insert(name, HealthStatus::Stopped);
                    continue;
                }
            };
            if !self.is_deps_ready(&factory.deps) {
                eprintln!("依赖未准备完成, 跳过初始化");
                self.health.services.insert(name, HealthStatus::Stopped);
                continue;
            }
            let endpoint = match self.get_endpoint(name).await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    eprintln!("无法获取 Endpoint: {}", e);
                    self.health.services.insert(name, HealthStatus::Stopped);
                    continue;
                }
            };
            let handle = match (factory.launch)(endpoint) {
                Ok(handle) => handle,
                Err(e) => {
                    eprintln!("初始化服务失败: {}", e);
                    self.health.services.insert(name, HealthStatus::Stopped);
                    continue;
                }
            };
            self.send_admin_message(AdminCommand::AddService(handle))
                .await;
            self.health.services.insert(name, HealthStatus::Healthy);
            println!("初始化服务成功: {}", name);
        }
    }

    /// 终止各个服务
    async fn stop_all_services(&mut self) {
        let order = match &self.order {
            Some(Ok(order)) => order,
            _ => {
                self.send_admin_message(AdminCommand::Shutdown(
                    crate::command::ShutdownStage::StopKernel,
                ))
                .await;
                return;
            }
        };
        for &name in order.iter().rev() {
            match self.health.services.get(name) {
                Some(HealthStatus::Healthy) => {
                    self.health.services.insert(name, HealthStatus::Stopping)
                }
                _ => continue,
            };
            let (tx, rx) = oneshot::channel();
            let _ = self
                .endpoint
                .send(name, Box::new(CommonMessage::Stop(tx)))
                .await;
            match rx.await {
                Ok(_) => {
                    self.send_admin_message(AdminCommand::DeleteService(name))
                        .await;
                }
                Err(e) => {
                    eprintln!("获取关闭 {} 服务反馈失败: {}", name, e);
                }
            }
            self.health.services.insert(name, HealthStatus::Stopped);
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
        deps.iter()
            .all(|&name| self.health.services.get(name) == Some(&HealthStatus::Healthy))
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
