

pub enum HealthStatus {
    Starting,
    Healthy,
    Unhealthy(String),
    Dead,
}

pub struct KernelHealth {
    kernel: HealthStatus,
}

impl KernelHealth {
    pub fn new()->Self{
        Self { kernel: HealthStatus::Starting }
    }
}