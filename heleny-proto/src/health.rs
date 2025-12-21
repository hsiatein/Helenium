use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

#[derive(PartialEq, Clone, Debug)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Hung,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug)]
pub struct KernelHealth {
    pub kernel: HealthStatus,
    pub services: HashMap<&'static str, HealthStatus>,
}

impl KernelHealth {
    pub fn get_mut<'a>(health: &'a Arc<Mutex<KernelHealth>>) -> MutexGuard<'a, KernelHealth> {
        health.as_ref().lock().expect("无法获取 health 锁")
    }
}
