use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Local;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::{message::{AnyMessage}, service_handle::ServiceHandle};
use heleny_proto::{
    common_message::CommonMessage, kernel_service_message::KernelServiceMessage,
    kernel_service_message::ServiceSignal, message::SignedMessage, name::KERNEL_NAME,
    role::ServiceRole,
};
use heleny_service::{Service, ServiceFactory, ServiceFactoryVec};
use inventory;
use tokio::{sync::oneshot, time::{Instant, timeout}};

use crate::command::AdminCommand;
use heleny_proto::health::HealthStatus;
use heleny_proto::health::KernelHealth;
use tracing::{debug, info, warn};

mod cal_deps;

#[base_service(deps=[])]
pub struct KernelService {
    endpoint: Endpoint,
    service_factories: Vec<ServiceFactoryVec>,
    deps_relation: cal_deps::DepsRelation,
    services: Arc<Mutex<HashMap<&'static str, ServiceHandle>>>,
    health: Arc<Mutex<KernelHealth>>,
    is_waiting: HashMap<&'static str, Vec<oneshot::Sender<Result<()>>>>,
}

#[async_trait]
impl Service for KernelService {
    type MessageType = KernelServiceMessage;
    async fn new(mut endpoint: Endpoint) -> Result<Box<Self>> {
        // 初始化服务工厂
        let service_factories: Vec<ServiceFactoryVec> = inventory::iter::<ServiceFactory>
            .into_iter()
            .map(|ServiceFactory { name, deps, launch }| {
                let mut deps = Vec::from(*deps);
                if *name != KernelService::name() && !deps.contains(&KernelService::name()) {
                    deps.push(KernelService::name());
                }
                ServiceFactoryVec {
                    name,
                    deps,
                    launch: launch.clone(),
                }
            })
            .collect();
        // 计算服务依赖
        let dag_map = service_factories
            .iter()
            .map(|f| {
                (
                    f.name,
                    f.deps.iter().copied().collect::<HashSet<&'static str>>(),
                )
            })
            .collect::<HashMap<&'static str, HashSet<&'static str>>>();
        let deps_relation = cal_deps::DepsRelation::new(dag_map.clone())?;
        // 构建健康表
        let (health, services) = match endpoint.recv().await {
            Ok(SignedMessage {
                target: _,
                name: _,
                role: _,
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
                Ok(Err(msg)) => {
                    return Err(anyhow::anyhow!(
                        "收到初始化参数, 但是解析为 CommonMessage: {:?}",
                        msg
                    ));
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("收到初始化参数, 但是解析失败: {}", e));
                }
            },
            Err(e) => {
                return Err(anyhow::anyhow!("未收到初始化参数: {}",e));
            }
        };
        KernelHealth::get_mut(&health)
            .services
            .insert(Self::name(), (HealthStatus::Healthy, Some(Local::now())));
        // 构建完成
        Ok(Box::new(Self {
            endpoint,
            service_factories,
            deps_relation,
            services,
            health,
            is_waiting:HashMap::new(),
        }))
    }
    async fn handle(
        &mut self,
        name: &'static str,
        role: ServiceRole,
        msg: Box<KernelServiceMessage>,
    ) -> anyhow::Result<()> {
        match (*msg, role) {
            (KernelServiceMessage::StopAll(sender), ServiceRole::System) => {
                let _ = sender.send(());
                let can_stop = self
                    .deps_relation
                    .prepare_all_services(KernelHealth::get_mut(&self.health).to_owned(), false)?;
                if can_stop.contains(KernelService::name()) {
                    self.send_admin_message(AdminCommand::Shutdown(
                        crate::command::ShutdownStage::StopKernel,
                    ))
                    .await;
                }
                else {
                    self.stop_services(can_stop).await;
                }
            }
            (KernelServiceMessage::Init, ServiceRole::System) => {
                let can_init = self
                    .deps_relation
                    .prepare_all_services(KernelHealth::get_mut(&self.health).to_owned(), true)?;
                self.init_services(can_init).await;
            }
            (KernelServiceMessage::GetHealth(sender), _) => {
                let _ = sender.send(KernelHealth::get_mut(&self.health).to_owned());
            }
            (KernelServiceMessage::UploadStatus(status), _) => match status {
                ServiceSignal::Alive => {
                    self.notify(name);
                    KernelHealth::get_mut(&self.health).set_alive(name);
                }
                ServiceSignal::InitFail => {
                    KernelHealth::get_mut(&self.health).set_dead(name);
                    let mut services = match self.services.as_ref().lock() {
                        Ok(service) => service,
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "无法获取 {} 的锁, 导致无法 Abort: {}",
                                name,
                                e
                            ));
                        }
                    };
                    services
                        .get(name)
                        .context(format!("未找到 {} 的句柄, 导致无法 Abort", name))?
                        .abort();
                    services.remove(name);
                }
                ServiceSignal::Ready => {
                    self.notify(name);
                    if name == Self::name() {
                        return Ok(());
                    }
                    info!("{} 成功初始化", name);
                    KernelHealth::get_mut(&self.health).set_alive(name);
                    let can_init = self.deps_relation.refresh_cache(name, true)?;
                    if !can_init.is_empty() {
                        self.init_services(can_init).await;
                    }
                }
                ServiceSignal::Terminate => {
                    info!("{} 成功退出", name);
                    KernelHealth::get_mut(&self.health).set_dead(name);
                    {
                        let mut services = match self.services.as_ref().lock() {
                            Ok(service) => service,
                            Err(e) => {
                                return Err(anyhow::anyhow!(
                                    "无法获取 {} 的锁, 导致无法清理: {}",
                                    name,
                                    e
                                ));
                            }
                        };
                        services
                            .get(name)
                            .context(format!("未找到 {} 的句柄, 导致无法清理", name))?
                            .abort();
                        services.remove(name);
                    }
                    let can_stop = self.deps_relation.refresh_cache(name, false)?;
                    if can_stop.contains(KernelService::name()) {
                        self.send_admin_message(AdminCommand::Shutdown(
                            crate::command::ShutdownStage::StopKernel,
                        ))
                        .await;
                    } else if !can_stop.is_empty() {
                        self.stop_services(can_stop).await;
                    }
                }
            },
            (KernelServiceMessage::WaitFor { name, sender },_)=>{
                debug!("开始等待: {}",name);
                let health=match KernelHealth::get_mut(&self.health).services.get(name){
                    Some(service)=>{
                        service.0.clone()
                    }
                    None=>{
                        let _=sender.send(Err(anyhow::anyhow!("没有这个服务")));
                        return Err(anyhow::anyhow!("等待 {} 失败: 没有这个服务",name));
                    }
                };
                match health {
                    HealthStatus::Healthy=>{
                        let _=sender.send(Ok(()));
                    }
                    _=>{
                        match self.is_waiting.get_mut(name) {
                            Some(set)=>{
                                set.push(sender)
                            }
                            None=>{
                                self.is_waiting.insert(name, vec![sender]);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    async fn stop(&mut self) {
        info!("{} 已关闭", Self::name())
    }

    async fn handle_sub_endpoint(&mut self, _msg:Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
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

    /// 初始化一个列表里的各个服务
    async fn init_services(&mut self, can_init: HashSet<&'static str>) {
        info!("开始代理启动服务");
        for name in can_init {
            {
                let mut health = KernelHealth::get_mut(&self.health);
                let service_health = match health.services.get_mut(name) {
                    Some(service_health) => service_health,
                    None => {
                        warn!("未找到 {}, 忽略", name);
                        continue;
                    }
                };
                *service_health = match service_health {
                    (HealthStatus::Healthy, _) => {
                        info!("{} 已经是健康状态, 跳过初始化", name);
                        continue;
                    }
                    _ => {
                        info!("开始代理启动 {}", name);
                        (HealthStatus::Starting, Some(Local::now()))
                    }
                };
            }
            let factory = match self.service_factories.iter().find(|f| f.name == name) {
                Some(factory) => factory,
                None => {
                    warn!("未找到 {} 的工厂函数, 忽略", name);
                    KernelHealth::get_mut(&self.health)
                        .services
                        .insert(name, (HealthStatus::Stopped, Some(Local::now())));
                    continue;
                }
            };
            if !self.is_deps_ready(&factory.deps) {
                warn!("{} 依赖未准备完成, 跳过初始化", name);
                KernelHealth::get_mut(&self.health)
                    .services
                    .insert(name, (HealthStatus::Stopped, Some(Local::now())));
                continue;
            }
            let endpoint = match self.get_endpoint(name).await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    warn!("无法获取 Endpoint, {} 启动失败: {}", name, e);
                    KernelHealth::get_mut(&self.health)
                        .services
                        .insert(name, (HealthStatus::Stopped, Some(Local::now())));
                    continue;
                }
            };
            let handle = match (factory.launch)(endpoint) {
                Ok(handle) => handle,
                Err(e) => {
                    warn!("初始化 {} 失败: {}", name, e);
                    KernelHealth::get_mut(&self.health)
                        .services
                        .insert(name, (HealthStatus::Stopped, Some(Local::now())));
                    continue;
                }
            };
            self.services
                .as_ref()
                .lock()
                .expect("获取 services 锁失败")
                .insert(handle.name(), handle);
            info!("新建 {} 初始化过程完成", name);
        }
    }

    /// 终止一个列表里的各个服务
    async fn stop_services(&mut self, can_stop: HashSet<&'static str>) {
        info!("开始代理关闭服务");
        for name in can_stop {
            info!("开始代理关闭 {}", name);
            {
                let mut health = KernelHealth::get_mut(&self.health);
                match health.services.get(name) {
                    Some((HealthStatus::Healthy, _)) => health
                        .services
                        .insert(name, (HealthStatus::Stopping, Some(Local::now()))),
                    _ => {
                        warn!("{} 非健康状态, 强制杀死", name);
                        match self
                            .services
                            .as_ref()
                            .lock()
                            .expect("获取 services 锁失败")
                            .remove(name)
                        {
                            Some(handle) => {
                                handle.abort();
                                health
                                    .services
                                    .insert(name, (HealthStatus::Stopped, Some(Local::now())));
                                info!("强制终止 {} 句柄", name);
                            }
                            None => (),
                        };
                        continue;
                    }
                };
            }
            let _ = self
                .endpoint
                .send(name, Box::new(CommonMessage::Stop))
                .await;
        }
    }

    /// 向内核发送管理员消息
    async fn send_admin_message(&self, payload: AdminCommand) {
        let _ = self.endpoint.send(KERNEL_NAME, Box::new(payload)).await;
    }

    /// 检测前置服务是否准备好
    fn is_deps_ready(&self, deps: &Vec<&'static str>) -> bool {
        deps.iter().all(|&name| {
            KernelHealth::get_mut(&self.health)
                .services
                .get(name)
                .is_some_and(|(status, _)| *status == HealthStatus::Healthy)
        })
    }

    /// 
    fn notify(&mut self, name:&'static str){
        if let Some(set) = self.is_waiting.remove(name) {
            set.into_iter().for_each(|sender|{
                let _=sender.send(Ok(()));
            });
        }
    }
}
