use crate::command::{self, AdminCommand, ShutdownStage};
use crate::health::new_kernel_health;
use crate::service::{KernelService, KernelServiceMessage};
use anyhow::{Result, anyhow};
use heleny_bus::{self, Bus, Endpoint};
use heleny_proto::common_message::CommonMessage;
use heleny_proto::health::KernelHealth;
use heleny_proto::kernel_message::KernelMessage;
use heleny_proto::message::Message;
use heleny_proto::name::KERNEL_NAME;
use heleny_service::{HasName, ServiceHandle, get_factory};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{MissedTickBehavior, interval};
use uuid::Uuid;
use tracing::{debug, info, warn};

pub struct Kernel {
    bus: Bus,
    endpoint: Endpoint,
    services: Arc<Mutex<HashMap<&'static str, ServiceHandle>>>,
    health: Arc<Mutex<KernelHealth>>,
    admin_tokens: HashSet<Uuid>,

    service_buffer: usize,
    run: bool,
    time_tick: usize,
}

impl Kernel {
    pub async fn new(kernel_buffer: usize, service_buffer: usize) -> Result<Self> {
        let mut bus = Bus::new(kernel_buffer);
        let token = Uuid::new_v4();
        let endpoint = bus.get_token_endpoint(KERNEL_NAME, service_buffer, token);
        let mut kernel = Self {
            bus,
            endpoint,
            services: Arc::new(Mutex::new(HashMap::new())),
            health: Arc::new(Mutex::new(new_kernel_health())),
            admin_tokens: HashSet::from([token]),
            service_buffer,
            run: true,
            time_tick: 0,
        };
        match kernel.init_necessary_services().await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("创建 Kernel 失败, 因为必要服务启动失败: {}", e));
            }
        }
        Ok(kernel)
    }

    /// 运行内核
    pub async fn run(&mut self) {
        // 开始初始化服务
        let _ = self.init_all_services().await;
        // 计时器
        let mut tick_interval = interval(Duration::from_secs(1));
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        while self.run {
            tokio::select! {
                Some(msg) = self.bus.recv() => {
                    debug!("{:?}",msg);
                    if msg.target == KernelService::name(){
                        warn!("拒绝外部对私有服务 KernelService 的访问");
                    }
                    else if msg.target == KERNEL_NAME {
                        let command=match command::downcast(msg.payload) {
                            Ok(command) => command,
                            Err(e) => {
                                warn!("解析失败, 忽略命令: {}",e);
                                continue;
                            }
                        };
                        match command {
                            Ok(command) => {
                                if !self.verify_admin_token(msg.token) {
                                    warn!("无管理员权限, 忽略命令");
                                    continue;
                                }
                                self.handle_admin(*command).await;
                            }
                            Err(command) => self.handle(*command).await,
                        };
                    }
                    else {
                        if let Err(e) = self.bus.send(msg).await {
                            warn!("Kernel 发送消息时出错: {}", e);
                        }
                    }
                }
                _ = tick_interval.tick() => {
                    self.handle_tick().await;
                }
            }
        }
    }

    /// 初始化必要的服务
    async fn init_necessary_services(&mut self) -> Result<()> {
        let _ = self
            .init_service(
                KernelService::name(),
                true,
                Some(Message::new(
                    KernelService::name(),
                    None,
                    Box::new(KernelServiceMessage::InitParams(
                        self.health.clone(),
                        self.services.clone(),
                    )),
                )),
            )
            .await?;
        Ok(())
    }

    /// 初始化所有服务
    async fn init_all_services(&mut self) -> Result<()> {
        self.send_kernel_message(KernelServiceMessage::Init).await;
        Ok(())
    }

    /// 初始化一个服务
    async fn init_service(
        &mut self,
        name: &'static str,
        admin: bool,
        init_message: Option<Message>,
    ) -> Result<()> {
        let endpoint = match admin {
            true => {
                let token = self.generate_admin_token();
                self.bus
                    .get_token_endpoint(name, self.service_buffer, token)
            }
            false => self.bus.get_endpoint(name, self.service_buffer),
        };
        if let Some(msg) = init_message {
            let _ = self.bus.send(msg).await;
        }
        let f = match get_factory(name) {
            Some(f) if f.deps.len() == 0 => f,
            Some(_) => {
                return Err(anyhow::anyhow!("内核不能直接初始化有依赖的服务: {}", name));
            }
            None => {
                return Err(anyhow::anyhow!("未找到此服务: {}", name));
            }
        };
        let handle = match (f.launch)(endpoint) {
            Ok(handle) => handle,
            Err(e) => {
                return Err(anyhow::anyhow!("{} 服务初始化失败: {}", name, e));
            }
        };
        self.services
            .as_ref()
            .lock()
            .expect("获取 services 锁失败")
            .insert(handle.name(), handle);
        Ok(())
    }

    /// 生成管理员 token
    fn generate_admin_token(&mut self) -> Uuid {
        let token = Uuid::new_v4();
        self.admin_tokens.insert(token);
        token
    }

    /// 验证管理员 token
    fn verify_admin_token(&self, token: Option<Uuid>) -> bool {
        let token = match token {
            Some(token) => token,
            None => return false,
        };
        self.admin_tokens.contains(&token)
    }

    /// 发送消息给 KernelService
    async fn send_kernel_message(&self, payload: KernelServiceMessage) {
        let message = Message::new(KernelService::name(), None, Box::new(payload));
        let _ = self.bus.send(message).await;
    }

    /// 发送 Admin 消息给 Kernel(自己)
    async fn send_admin_command(&self, payload: AdminCommand) {
        let _ = self.endpoint.send(KERNEL_NAME, Box::new(payload)).await;
    }

    /// 发送消息给 Kernel(自己)
    async fn send_kernel_command(&self, payload: KernelMessage) {
        let _ = self.endpoint.send(KERNEL_NAME, Box::new(payload)).await;
    }

    // 关机
    async fn shutdown(&mut self, stage: ShutdownStage) {
        match stage {
            ShutdownStage::Start => {
                info!("开始关机");
                self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::StopAllService))
                    .await;
            }
            ShutdownStage::StopAllService => {
                info!("开始关闭所有服务");
                self.send_kernel_message(KernelServiceMessage::StopAll)
                    .await;
            }
            ShutdownStage::StopKernel => {
                info!("开始关闭内核");
                let (tx, rx) = oneshot::channel();
                let _ = self
                    .bus
                    .send_common_message(KernelService::name(), CommonMessage::Stop(tx))
                    .await;
                match rx.await {
                    Ok(_) => (),
                    Err(e) => warn!("关闭 KernelService 时发生错误: {}", e),
                };
                self.run = false;
            }
        }
    }

    /// 处理管理员 Command
    async fn handle_admin(&mut self, command: AdminCommand) {
        match command {
            AdminCommand::NewEndpoint(name, sender) => {
                let endpoint = self.bus.get_endpoint(name, self.service_buffer);
                let _ = sender.send(endpoint);
            }
            AdminCommand::Shutdown(stage) => {
                self.shutdown(stage).await;
            }
        }
    }

    /// 处理普通 Command
    async fn handle(&mut self, command: KernelMessage) {
        match command {
            KernelMessage::Shutdown => {
                self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::Start))
                    .await;
            }
            KernelMessage::GetHealth(sender) => {
                let _ = sender.send(KernelHealth::get_mut(&self.health).to_owned());
            }
        }
    }

    /// 处理 Tick
    async fn handle_tick(&mut self) {
        self.time_tick = self.time_tick + 1;
        if self.time_tick > 1 {
            self.send_kernel_command(KernelMessage::Shutdown).await;
        }
        debug!("{:?}", KernelHealth::get_mut(&self.health))
    }
}
