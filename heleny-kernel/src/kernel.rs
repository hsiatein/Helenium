use std::collections::HashMap;

use crate::bus::{self, Bus};
use crate::service::{ServiceHandle, Service};
use crate::kernel_service::KernelService;

pub struct Kernel {
    pub bus: Bus,
    pub services:Vec<ServiceHandle>,
}

impl Kernel {
    pub fn new() -> Self {
        let bus = bus::Bus::new(64);
        Self { bus, services: Vec::new() }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.bus.recv().await {
            if let Err(e) = self.bus.send(msg).await {
                eprintln!("Kernel 发送消息时出错: {}", e);
            }
        }
    }

    pub fn init_service(&mut self) {
        let mut dependencies_table:HashMap<&str, Vec<&str>>=HashMap::new();
    }
}
