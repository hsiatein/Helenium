use std::collections::HashMap;
use std::collections::VecDeque;

use chrono::DateTime;
use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::KernelHealth;
use crate::ScheduledTask;
use crate::TaskAbstract;
use crate::ToolAbstract;
use crate::memory::MemoryEntry;

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
pub static TASK_ABSTRACT: &'static str = "TaskAbstract";
pub static SCHEDULE: &'static str = "Schedule";
pub static TOOL_ABSTRACTS: &'static str = "ToolAbstracts";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePayload {
    Health(KernelHealth),
    TotalBusTraffic(VecDeque<(DateTime<Local>, usize)>),
    DisplayMessages {
        new: bool,
        messages: Vec<MemoryEntry>,
    },
    Image {
        id: i64,
        base64: String,
    },
    TaskAbstract {
        task_abstracts: Vec<TaskAbstract>,
    },
    TaskLogs {
        id: uuid::Uuid,
        logs: Vec<String>,
    },
    Schedules {
        schedules: HashMap<Uuid, ScheduledTask>,
    },
    ToolAbstracts {
        abstracts: Vec<ToolAbstract>,
    },
}
