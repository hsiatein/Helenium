use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};
use heleny_bus::{self, Bus};
use heleny_proto::message::AnyMessage;
use crate::command::KernelCommand;
use crate::health::{KernelHealth};
use crate::service::KernelService;
use heleny_service::{HasName, Service, ServiceHandle};
use uuid::Uuid;

pub struct Kernel {
    bus: Bus,
    services: HashMap<&'static str,ServiceHandle>,
    admin_tokens: HashSet<Uuid>,

    service_buffer: usize,
    health: KernelHealth,
}

impl Kernel {
    pub async fn new(kernel_buffer: usize, service_buffer: usize) -> Result<Self> {
        let bus = Bus::new(kernel_buffer);
        let mut kernel=Self { bus, services: HashMap::new(), admin_tokens: HashSet::new(),service_buffer , health: KernelHealth::new() };
        match kernel.init_necessary_service().await {
            Ok(_) => (),
            Err(e) => {
                return Err(anyhow!("创建 Kernel 失败, 因为必要服务启动失败: {}",e));
            }
        }
        Ok(kernel)
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.bus.recv().await{
            if msg.target=="Kernel" {
                if !self.verify_admin_token(msg.token) {
                    eprintln!("无管理员权限, 忽略命令");
                    continue;
                }
                let command=match KernelCommand::downcast(msg.payload) {
                    Ok(command) => command,
                    Err(e) => {
                        eprintln!("解析 KernelCommand失败, 忽略命令: {}",e);
                        continue;
                    }
                };
                match *command {
                    KernelCommand::Shutdown =>{

                    }
                    KernelCommand::AddService(handle) =>{
                        self.update_service(handle);
                    }
                }
            }
            else if let Err(e) = self.bus.send(msg).await {
                eprintln!("Kernel 发送消息时出错: {}", e);
            }
        }
    }

    async fn init_necessary_service(&mut self)->Result<()> {
        let token=self.generate_admin_token();
        let endpoint=self.bus.get_token_endpoint(KernelService::name(), self.service_buffer, token);
        let handle=KernelService::start(endpoint).await;
        let handle = match handle {
            Ok(handle) => handle,
            Err(e) => {
                return Err(anyhow::anyhow!("KernelService 内核服务初始化失败: {}",e));
            }
        };
        self.services.insert(handle.name(),handle);
        Ok(())
    }

    fn update_service(&mut self, handle: ServiceHandle) {
        self.services.insert(handle.name(), handle);
    }

    fn generate_admin_token(&mut self)-> Uuid {
        let token=Uuid::new_v4();
        self.admin_tokens.insert(token);
        token
    }

    fn verify_admin_token(&self, token:Option<Uuid>)-> bool {
        let token=match token {
            Some(token) => token,
            None => return false,
        };
        self.admin_tokens.contains(&token)
    }

    fn shutdown(&mut self) {
        
    }
}
