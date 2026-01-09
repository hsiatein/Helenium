use anyhow::Result;
use async_trait::async_trait;
use chrono::Local;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::HealthStatus;
use heleny_proto::KERNEL_NAME;
use heleny_proto::KERNEL_SERVICE;
use heleny_proto::KernelHealth;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::ServiceHandle;
use heleny_proto::ServiceRole;
use heleny_proto::SignedMessage;
use heleny_service::AdminCommand;
use heleny_service::CommonMessage;
use heleny_service::KernelServiceMessage;
use heleny_service::Service;
use heleny_service::ServiceFactory;
use heleny_service::ServiceFactoryVec;
use heleny_service::ShutdownStage;
use inventory;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::time::Instant;
use tracing::debug;
use tracing::info;
use tracing::warn;

mod cal_deps;
mod handle_status;

#[base_service(deps=[])]
pub struct KernelService {
    endpoint: Endpoint,
    service_factories: Vec<ServiceFactoryVec>,
    deps_relation: cal_deps::DepsRelation,
    services: Arc<Mutex<HashMap<String, ServiceHandle>>>,
    health: Arc<Mutex<KernelHealth>>,
    is_waiting: HashMap<String, Vec<oneshot::Sender<Result<()>>>>,
    health_tx: Option<watch::Sender<ResourcePayload>>,
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
                if *name != KERNEL_SERVICE && !deps.contains(&KERNEL_SERVICE) {
                    deps.push(KERNEL_SERVICE);
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
                    f.name.to_string(),
                    f.deps
                        .iter()
                        .map(|str| str.to_string())
                        .collect::<HashSet<String>>(),
                )
            })
            .collect::<HashMap<String, HashSet<String>>>();
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
                return Err(anyhow::anyhow!("未收到初始化参数: {}", e));
            }
        };
        KernelHealth::get_mut(&health).services.insert(
            KERNEL_SERVICE.to_string(),
            (HealthStatus::Healthy, Some(Local::now())),
        );
        // 构建完成
        Ok(Box::new(Self {
            endpoint,
            service_factories,
            deps_relation,
            services,
            health,
            is_waiting: HashMap::new(),
            health_tx: None,
        }))
    }
    async fn handle(
        &mut self,
        name: String,
        role: ServiceRole,
        msg: KernelServiceMessage,
    ) -> anyhow::Result<()> {
        match (msg, role) {
            (KernelServiceMessage::StopAll(sender), ServiceRole::System) => {
                let _ = sender.send(());
                let can_stop = self
                    .deps_relation
                    .prepare_all_services(KernelHealth::get_mut(&self.health).to_owned(), false)?;
                if can_stop.contains(KernelService::name()) {
                    self.send_admin_message(AdminCommand::Shutdown(ShutdownStage::StopKernel))
                        .await;
                } else {
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
            (KernelServiceMessage::UploadStatus(status), _) => {
                self.handle_status(status, name).await?
            }
            (KernelServiceMessage::WaitFor { name, sender }, _) => {
                debug!("开始等待: {}", name);
                let health = match KernelHealth::get_mut(&self.health).services.get(&name) {
                    Some(service) => service.0.clone(),
                    None => {
                        let _ = sender.send(Err(anyhow::anyhow!("没有这个服务")));
                        return Err(anyhow::anyhow!("等待 {} 失败: 没有这个服务", name));
                    }
                };
                match health {
                    HealthStatus::Healthy => {
                        let _ = sender.send(Ok(()));
                    }
                    _ => match self.is_waiting.get_mut(&name) {
                        Some(set) => set.push(sender),
                        None => {
                            self.is_waiting.insert(name, vec![sender]);
                        }
                    },
                }
            }
            _ => {}
        }
        Ok(())
    }
    async fn stop(&mut self) {
        info!("{} 已关闭", Self::name())
    }

    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl KernelService {
    /// 初始化一个列表里的各个服务
    async fn init_services(&mut self, can_init: HashSet<String>) {
        for name in can_init {
            {
                let mut health = KernelHealth::get_mut(&self.health);
                let service_health = match health.services.get_mut(&name) {
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
                    KernelHealth::get_mut(&self.health).services.insert(
                        name.to_string(),
                        (HealthStatus::Stopped, Some(Local::now())),
                    );
                    continue;
                }
            };
            if !self.is_deps_ready(&factory.deps) {
                warn!("{} 依赖未准备完成, 跳过初始化", name);
                KernelHealth::get_mut(&self.health).services.insert(
                    name.to_string(),
                    (HealthStatus::Stopped, Some(Local::now())),
                );
                continue;
            }
            let endpoint = match self.get_endpoint_from_kernel(&name).await {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    warn!("无法获取 Endpoint, {} 启动失败: {}", name, e);
                    KernelHealth::get_mut(&self.health).services.insert(
                        name.to_string(),
                        (HealthStatus::Stopped, Some(Local::now())),
                    );
                    continue;
                }
            };
            let handle = match (factory.launch)(endpoint) {
                Ok(handle) => handle,
                Err(e) => {
                    warn!("初始化 {} 失败: {}", name, e);
                    KernelHealth::get_mut(&self.health).services.insert(
                        name.to_string(),
                        (HealthStatus::Stopped, Some(Local::now())),
                    );
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
    async fn stop_services(&mut self, can_stop: HashSet<String>) {
        for name in can_stop {
            info!("开始代理关闭 {}", name);
            let mut killed = false;
            {
                let mut health = KernelHealth::get_mut(&self.health);
                match health.services.get(&name) {
                    Some((HealthStatus::Healthy, _)) => {
                        let _ = health.services.insert(
                            name.to_string(),
                            (HealthStatus::Stopping, Some(Local::now())),
                        );
                    }
                    _ => {
                        warn!("{} 非健康状态, 强制杀死", name);
                        {
                            match self
                                .services
                                .as_ref()
                                .lock()
                                .expect("获取 services 锁失败")
                                .remove(&name)
                            {
                                Some(handle) => {
                                    handle.abort();
                                    health.services.insert(
                                        name.to_string(),
                                        (HealthStatus::Stopped, Some(Local::now())),
                                    );
                                    info!("强制终止 {} 句柄", name);
                                }
                                None => (),
                            };
                        }
                        killed = true;
                    }
                };
            }
            if killed {
                let _ = self
                    .endpoint
                    .send(
                        KERNEL_SERVICE,
                        KernelServiceMessage::UploadStatus(
                            heleny_service::ServiceSignal::Terminate(name),
                        ),
                    )
                    .await;
                continue;
            }
            let _ = self.endpoint.send(&name, CommonMessage::Stop).await;
        }
    }

    /// 向内核发送管理员消息
    async fn send_admin_message(&self, payload: AdminCommand) {
        let _ = self.endpoint.send(KERNEL_NAME, payload).await;
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
    fn notify(&mut self, name: &str) {
        if let Some(set) = self.is_waiting.remove(name) {
            set.into_iter().for_each(|sender| {
                let _ = sender.send(Ok(()));
            });
        }
    }
}
