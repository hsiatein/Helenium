use std::path::PathBuf;

use crate::UserDecision;
use crate::resource::Resource;
use serde::Deserialize;
use serde::Serialize;
use tungstenite::Message;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrontendMessage {
    UpdateResource(Resource),
    UserDecision(UserDecision),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrontendCommand {
    UserInput(String),
    GetHistory(i64),
    GetHealth,
    Shutdown,
    Close,
    GetImage { id: i64, path: PathBuf },
    MakeDecision { req_id: Uuid, approval: bool },
    GetConsentRequestions,
    CancelTask { id: Uuid },
    CancelSchedule { id: Uuid },
    ToggleTaskLogs { id: Uuid, expanded: bool },
    GetSchedules,
    GetToolAbstrats,
    ReloadTools,
    EnableTool {
        name:String,
        enable:bool,
    },
    DeleteMemory {
        id: i64,
    }
}

impl FrontendCommand {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or(String::new())
    }
}

impl From<FrontendCommand> for Message {
    fn from(value: FrontendCommand) -> Self {
        value.to_string().into()
    }
}
