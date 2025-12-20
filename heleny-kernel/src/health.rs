

pub enum HealthStatus {
    Starting,
    Healthy,
    Degraded,
    Unhealthy(String),
    Dead,
}

pub struct KernelHealth {
    kernel_health: bool,
}