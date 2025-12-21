use std::collections::HashMap;

#[derive(PartialEq, Clone, Debug)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug)]
pub struct KernelHealth {
    pub kernel: HealthStatus,
    pub services: HashMap<&'static str, HealthStatus>,
}
