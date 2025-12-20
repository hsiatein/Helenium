use std::collections::HashMap;

use crate::bus::{self, Bus};
use crate::health::KernelHealth;
use crate::kernel_service::KernelService;
use crate::service::{HasName, Service, ServiceHandle};

pub struct Kernel {
    bus: Bus,
    services: Vec<ServiceHandle>,

    kernel_buffer: usize,
    service_buffer: usize,
    health: KernelHealth,
}

impl Kernel {
    pub async fn new(kernel_buffer: usize, service_buffer: usize) -> Self {
        let bus = bus::Bus::new(kernel_buffer);
        let mut kernel=Self { bus, services: Vec::new(), kernel_buffer, service_buffer };
        kernel.init_service();
        kernel
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.bus.recv().await {
            if let Err(e) = self.bus.send(msg).await {
                eprintln!("Kernel 发送消息时出错: {}", e);
            }
        }
    }

    async fn init_service(&mut self) {
        let handle=KernelService::start(self.bus.get_endpoint(KernelService::name(), self.service_buffer)).await;
        let handle = match handle {
            Ok(handle) => handle,
            Err(e) => 
        };
        let mut dependencies_table:HashMap<&'static str, Vec<&'static str>>=HashMap::new();
        dependencies_table.insert(KernelService::name(), KernelService::dependencies());

    }

    fn add_service(&mut self) {

    }
}
