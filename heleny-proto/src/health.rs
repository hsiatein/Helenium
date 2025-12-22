use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::{Local};

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
    pub last_signal: HashMap<&'static str,chrono::prelude::DateTime<Local>>,
}

impl KernelHealth {
    pub fn get_mut<'a>(health: &'a Arc<Mutex<KernelHealth>>) -> MutexGuard<'a, KernelHealth> {
        health.as_ref().lock().expect("无法获取 health 锁")
    }

    pub fn update(&mut self){
        let now=Local::now();
        for (name, status) in &mut self.services {
            status=match self.last_signal.get(*name) {
                Some(time) =>{
                    let dt =  now-time;
                    dt.as_seconds_f32() < 5.
                }
                None => status,
            }
        }
        self.services.iter_mut().filter(|(name, status)| {
            match self.last_signal.get(*name) {
                Some(time) =>{
                    let dt =  now-time;
                    dt.as_seconds_f32() < 5.
                }
                None => false,
            }
        });
    }
}
