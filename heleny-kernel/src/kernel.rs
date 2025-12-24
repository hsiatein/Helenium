use crate::command::{self, AdminCommand, ShutdownStage};
use crate::health::new_kernel_health;
use crate::service::KernelService;
use anyhow::{Result, anyhow};
use heleny_bus::{self, BusHandle, endpoint::Endpoint};
use heleny_proto::common_message::CommonMessage;
use heleny_proto::health::KernelHealth;
use heleny_proto::kernel_message::KernelMessage;
use heleny_proto::kernel_service_message::KernelServiceMessage;
use heleny_proto::message::{AnyMessage, SignedMessage};
use heleny_proto::name::KERNEL_NAME;
use heleny_proto::role::ServiceRole;
use heleny_proto::service_handle::ServiceHandle;
use heleny_service::{HasName, get_factory};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{MissedTickBehavior, interval, timeout};
use tracing::{debug, info, warn};

pub struct Kernel {
    bus: BusHandle,
    endpoint: Endpoint,
    services: Arc<Mutex<HashMap<&'static str, ServiceHandle>>>,
    health: Arc<Mutex<KernelHealth>>,

    service_buffer: usize,
    run: bool,
    time_tick: usize,
}

impl Kernel {
    pub async fn new(kernel_buffer: usize, service_buffer: usize) -> Result<Self> {
        let mut bus = BusHandle::new(kernel_buffer);
        let endpoint = bus
            .get_endpoint(KERNEL_NAME, service_buffer, ServiceRole::System)
            .await?;
        let mut kernel = Self {
            bus,
            endpoint,
            services: Arc::new(Mutex::new(HashMap::new())),
            health: Arc::new(Mutex::new(new_kernel_health())),
            service_buffer,
            run: true,
            time_tick: 0,
        };
        if let Err(e) = kernel.init_necessary_services().await {
            return Err(anyhow!("创建 Kernel 失败, 因为必要服务启动失败: {}", e));
        }
        Ok(kernel)
    }

    /// 运行内核
    pub async fn run(&mut self) {
        // 开始初始化服务
        let _ = self.init_all_services().await;
        // 计时器
        let (mut from_bus,mut from_sub_endpoint)=self.endpoint.get_rx().expect("Kernel 应当获取到接收端");
        let mut tick_interval = interval(Duration::from_secs(1));
        tick_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        while self.run {
            tokio::select! {
                Some(msg) = from_bus.recv() => {
                    if let Err(e) = self.handle_msg(msg).await {
                        warn!("{}",e)
                    };
                }
                Some(msg) = from_sub_endpoint.recv() => {
                    if let Err(e) = self.handle_sub_endpoint(msg).await {
                        warn!("{}",e)
                    };
                }
                _ = tick_interval.tick() => {
                    if let Err(e) = self.handle_tick().await {
                        warn!("{}",e)
                    };
                }
            }
        }
    }

    /// 处理已签名消息
    async fn handle_msg(&mut self, msg: SignedMessage) -> Result<()> {
        let command = command::downcast(msg.payload)?;
        match command {
            Ok(command) => match msg.role {
                ServiceRole::System => self.handle_admin(*command, msg.name, msg.role).await,
                _ => Err(anyhow::anyhow!("无 System 权限, 忽略命令")),
            },
            Err(command) => self.handle(*command, msg.name, msg.role).await,
        }
    }

    /// 处理已签名消息
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }

    /// 初始化必要的服务
    async fn init_necessary_services(&mut self) -> Result<()> {
        let _ = self
            .init_service(
                KernelService::name(),
                ServiceRole::System,
                Some(Box::new(KernelServiceMessage::InitParams(
                    self.health.clone(),
                    self.services.clone(),
                ))),
            )
            .await?;
        Ok(())
    }

    /// 初始化所有服务
    async fn init_all_services(&mut self) -> Result<()> {
        self.send_kernel_message(KernelServiceMessage::Init).await
    }

    /// 初始化一个服务
    async fn init_service(
        &mut self,
        name: &'static str,
        role: ServiceRole,
        init_message: Option<Box<dyn AnyMessage>>,
    ) -> Result<()> {
        info!("初始化 {} 的 Endpoint", name);
        let endpoint = self
            .bus
            .get_endpoint(name, self.service_buffer, role)
            .await?;
        if let Some(msg) = init_message {
            info!("发送 {} 的初始化参数", name);
            let _ = self.endpoint.send(name, msg).await;
            // let a=endpoint.recv().await;
        }
        info!("寻找 {} 的工厂函数", name);
        let f = match get_factory(name) {
            Some(f) if f.deps.len() == 0 => f,
            Some(_) => {
                return Err(anyhow::anyhow!("内核不能直接初始化有依赖的服务: {}", name));
            }
            None => {
                return Err(anyhow::anyhow!("未找到此服务的工厂函数: {}", name));
            }
        };
        info!("启动 {}", name);
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

    /// 发送消息给 KernelService
    async fn send_kernel_message(&self, payload: KernelServiceMessage) -> Result<()> {
        self.endpoint
            .send(KernelService::name(), Box::new(payload))
            .await
    }

    /// 发送 Admin 消息给 Kernel(自己)
    async fn send_admin_command(&self, payload: AdminCommand) -> Result<()> {
        self.endpoint.send(KERNEL_NAME, Box::new(payload)).await
    }

    /// 发送消息给 Kernel(自己)
    async fn send_kernel_command(&self, payload: KernelMessage) -> Result<()> {
        self.endpoint.send(KERNEL_NAME, Box::new(payload)).await
    }

    // 关机
    async fn shutdown(&mut self, stage: ShutdownStage) -> Result<()> {
        match stage {
            ShutdownStage::Start => {
                info!("开始关机");
                self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::StopAllService))
                    .await
            }
            ShutdownStage::StopAllService => {
                info!("开始关闭所有服务");
                let (tx, rx) = oneshot::channel();
                let _ = self
                    .send_kernel_message(KernelServiceMessage::StopAll(tx))
                    .await;
                match timeout(Duration::from_secs(5), rx).await {
                    Ok(Ok(_)) => {
                        return Ok(());
                    }
                    Ok(Err(e)) => {
                        warn!("获取 KernelService 关闭所有服务反馈出错: {}", e);
                    }
                    Err(e) => {
                        warn!("获取 KernelService 关闭所有服务反馈超时: {}", e);
                    }
                }
                self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::StopKernel))
                    .await
            }
            ShutdownStage::StopKernel => {
                info!("开始关闭内核");
                let _ = self
                    .endpoint
                    .send(KernelService::name(), Box::new(CommonMessage::Stop))
                    .await;
                self.bus.abort();
                self.run = false;
                Ok(())
            }
        }
    }

    /// 处理管理员 Command
    async fn handle_admin(
        &mut self,
        command: AdminCommand,
        _: &'static str,
        _: ServiceRole,
    ) -> Result<()> {
        match command {
            AdminCommand::NewEndpoint(name, sender) => {
                let endpoint = self
                    .bus
                    .get_endpoint(name, self.service_buffer, ServiceRole::Standard)
                    .await?;

                let _ = sender.send(endpoint);
                Ok(())
            }
            AdminCommand::Shutdown(stage) => self.shutdown(stage).await,
        }
    }

    /// 处理普通 Command
    async fn handle(
        &mut self,
        command: KernelMessage,
        source: &'static str,
        role: ServiceRole,
    ) -> Result<()> {
        match command {
            KernelMessage::Shutdown => match role {
                ServiceRole::User => {
                    self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::Start))
                        .await
                }
                ServiceRole::System => {
                    self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::Start))
                        .await
                }
                _ => Err(anyhow::anyhow!(
                    "{} 的身份为 {:?}, 无关机权限",
                    source,
                    role
                )),
            },
        }
    }

    /// 处理 Tick
    async fn handle_tick(&mut self) -> Result<()> {
        self.time_tick = self.time_tick + 1;
        if self.time_tick == 2 {
            self.send_kernel_command(KernelMessage::Shutdown).await?;
        }
        debug!("{:?}", KernelHealth::get_mut(&self.health));
        Ok(())
    }
}
