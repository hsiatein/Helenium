use std::collections::VecDeque;

use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;

use crate::KernelHealth;
use crate::memory::DisplayMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub name: String,
    pub payload: ResourcePayload,
}

impl Resource {
    pub fn new(name: &str, payload: ResourcePayload) -> Self {
        Self {
            name: name.to_string(),
            payload,
        }
    }
}

pub static TOTAL_BUS_TRAFFIC: &'static str = "TotalBusTraffic";
pub static DISPLAY_MESSAGES: &'static str = "DisplayMessages";
pub static HEALTH: &'static str = "Health";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePayload {
    Health(KernelHealth),
    TotalBusTraffic(VecDeque<(DateTime<Local>, usize)>),
    DisplayMessages {
        new: bool,
        messages: Vec<DisplayMessage>,
    },
}
