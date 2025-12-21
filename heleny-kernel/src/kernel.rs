use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use heleny_bus::{self, Bus, Endpoint};
use heleny_proto::common_message::CommonMessage;
use heleny_proto::message::Message;
use heleny_proto::name::{KERNEL_NAME};
use tokio::sync::oneshot;
use crate::command::{self, AdminCommand, KernelCommand, ShutdownStage};
use crate::service::{KernelMessage, KernelService};
use heleny_service::{HasName, Service, ServiceHandle};
use uuid::Uuid;

pub struct Kernel {
    bus: Bus,
    endpoint: Endpoint,
    services: HashMap<&'static str,ServiceHandle>,
    admin_tokens: HashSet<Uuid>,

    service_buffer: usize,
    run: bool,
}

impl Kernel {
    pub async fn new(kernel_buffer: usize, service_buffer: usize) -> Result<Self> {
        let mut bus = Bus::new(kernel_buffer);
        let token=Uuid::new_v4();
        let endpoint=bus.get_token_endpoint(KERNEL_NAME, service_buffer, token);
        let mut kernel=Self { bus, endpoint, services: HashMap::new(), admin_tokens: HashSet::from([token]),service_buffer , run: true };
        match kernel.init_necessary_service().await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("创建 Kernel 失败, 因为必要服务启动失败: {}",e));
            }
        }
        Ok(kernel)
    }

    /// 运行内核
    pub async fn run(&mut self) {
        let _=self.init_all_service().await;
        while let Some(msg) = self.bus.recv().await{
            if !self.run {break;}
            if msg.target == KernelService::name(){
                eprintln!("拒绝外部对私有服务 KernelService 的访问");
            }
            else if msg.target == KERNEL_NAME {
                let command=match command::downcast(msg.payload) {
                    Ok(command) => command,
                    Err(e) => {
                        eprintln!("解析失败, 忽略命令: {}",e);
                        continue;
                    }
                };
                match command {
                    Ok(command) => {
                        if !self.verify_admin_token(msg.token) {
                            eprintln!("无管理员权限, 忽略命令");
                            continue;
                        }
                        self.handle_admin(*command).await;
                    }
                    Err(command) => self.handle(*command).await,
                };
            }
            else {
                if let Err(e) = self.bus.send(msg).await {
                    eprintln!("Kernel 发送消息时出错: {}", e);
                }
            }
        }
    }

    /// 初始化必要的服务
    async fn init_necessary_service(&mut self)->Result<()> {
        let token=self.generate_admin_token();
        let endpoint=self.bus.get_token_endpoint(KernelService::name(), self.service_buffer, token);
        let handle = match KernelService::start(endpoint) {
            Ok(handle) => handle,
            Err(e) => {
                return Err(anyhow::anyhow!("KernelService 内核服务初始化失败: {}",e));
            }
        };
        self.services.insert(handle.name(),handle);
        Ok(())
    }

    /// 初始化所有服务
    async fn init_all_service(&mut self)->Result<()> {
        self.send_kernel_message(KernelMessage::Init).await;
        Ok(())
    }

    /// 生成管理员 token
    fn generate_admin_token(&mut self)-> Uuid {
        let token=Uuid::new_v4();
        self.admin_tokens.insert(token);
        token
    }

    /// 验证管理员 token
    fn verify_admin_token(&self, token:Option<Uuid>)-> bool {
        let token=match token {
            Some(token) => token,
            None => return false,
        };
        self.admin_tokens.contains(&token)
    }

    /// 发送消息给 KernelService
    async fn send_kernel_message(&self,payload: KernelMessage){
        let message=Message::new(KernelService::name(), None, Box::new(payload));
        let _=self.bus.send(message).await;
    }

    /// 发送 Admin 消息给 Kernel(自己)
    async fn send_admin_command(&self,payload: AdminCommand){
        let _=self.endpoint.send(KERNEL_NAME,Box::new(payload)).await;
    }

    // 关机
    async fn shutdown(&mut self, stage:ShutdownStage) {
        match stage {
            ShutdownStage::Start=>{
                self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::StopAllService)).await;
            }
            ShutdownStage::StopAllService=>{
                self.send_kernel_message(KernelMessage::StopAll).await;
            }
            ShutdownStage::StopKernel=>{
                let (tx,rx)=oneshot::channel();
                let _=self.bus.send_common_message(KernelService::name(),CommonMessage::Stop(tx)).await;
                match rx.await {
                    Ok(_) => (),
                    Err(e) => eprintln!("关闭 KernelService 时发生错误: {}",e),
                };
                self.run=false;
            }
        }
        
    }

    /// 处理管理员 Command
    async fn handle_admin(&mut self, command: AdminCommand){
        match command {
            AdminCommand::AddService(handle) =>{
                self.services.insert(handle.name(), handle);
            }
            AdminCommand::DeleteService(name) =>{
                self.services.remove(name);
            }
            AdminCommand::NewEndpoint(name, sender) =>{
                let endpoint=self.bus.get_endpoint(name, self.service_buffer);
                let _=sender.send(endpoint);
            }
            AdminCommand::Shutdown(stage) =>{
                self.shutdown(stage).await;
            }
        }
    }

    /// 处理普通Command
    async fn handle(&mut self, command: KernelCommand){
        match command {
            KernelCommand::Shutdown =>{
                self.send_admin_command(AdminCommand::Shutdown(ShutdownStage::Start)).await;
            }
            KernelCommand::GetHealth =>{

            }
        }
    }
}
