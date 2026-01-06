use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use tracing::warn;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Unhealthy,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KernelHealth {
    pub kernel: HealthStatus,
    pub services: HashMap<String, (HealthStatus, Option<chrono::prelude::DateTime<Local>>)>,
}

impl KernelHealth {
    pub fn get_mut<'a>(health: &'a Arc<Mutex<KernelHealth>>) -> MutexGuard<'a, KernelHealth> {
        health.as_ref().lock().expect("无法获取 health 锁")
    }

    pub fn update(&mut self) {
        let now = Local::now();
        for (_, (status, last_signal)) in &mut self.services {
            match last_signal {
                Some(time) => {
                    if (now - *time).as_seconds_f64() > 5.0 && *status == HealthStatus::Healthy {
                        *status = HealthStatus::Unhealthy
                    }
                }
                None => *status = HealthStatus::Starting,
            }
        }
    }

    pub fn is_same(&self, other: &KernelHealth) -> bool {
        if self.kernel != other.kernel {
            return false;
        };
        if self.services.len() != other.services.len() {
            return false;
        };
        self.services.keys().all(
            |key| match (self.services.get(key), other.services.get(key)) {
                (Some(a), Some(b)) => a.0 == b.0,
                _ => false,
            },
        )
    }

    pub fn set_alive(&mut self, name: &str) {
        let (status, time) = match self.services.get_mut(name) {
            Some(s) => s,
            None => {
                warn!("未知服务: {}", name);
                return;
            }
        };
        *status = HealthStatus::Healthy;
        *time = Some(Local::now());
    }

    pub fn set_dead(&mut self, name: &str) {
        let (status, time) = match self.services.get_mut(name) {
            Some(s) => s,
            None => {
                warn!("未知服务: {}", name);
                return;
            }
        };
        *status = HealthStatus::Stopped;
        *time = Some(Local::now());
    }
}
