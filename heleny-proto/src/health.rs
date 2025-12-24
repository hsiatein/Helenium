use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::Local;
use tracing::warn;

#[derive(PartialEq, Clone, Debug)]
pub enum HealthStatus {
    Starting,
    Healthy,
    Unhealthy,
    Stopping,
    Stopped,
}

#[derive(Clone, Debug)]
pub struct KernelHealth {
    pub kernel: HealthStatus,
    pub services: HashMap<&'static str, (HealthStatus, Option<chrono::prelude::DateTime<Local>>)>,
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

    pub fn set_alive(&mut self, name: &'static str) {
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

    pub fn set_dead(&mut self, name: &'static str) {
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
