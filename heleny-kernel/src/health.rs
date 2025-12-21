use std::collections::{HashMap};

use heleny_service::{HasName, ServiceFactory};

use crate::service::KernelService;


#[derive(PartialEq)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Stopping,
    Stopped,
}

pub struct KernelHealth {
    pub kernel: HealthStatus,
    pub services: HashMap<&'static str,HealthStatus>,
}

impl KernelHealth {
    pub fn new()->Self{
        let services=inventory::iter::<ServiceFactory>.into_iter().filter(|f| f.name!=KernelService::name()).map(|ServiceFactory { name, deps:_, launch:_ }| (*name,HealthStatus::Starting)).collect();
        Self { kernel: HealthStatus::Starting, services }
    }
}