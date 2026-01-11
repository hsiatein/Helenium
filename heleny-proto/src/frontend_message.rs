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
    ToggleTaskLogs { id: Uuid, expanded: bool },
    GetSchedule,
}

impl FrontendCommand {
    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap_or(String::new())
    }
}

impl Into<Message> for FrontendCommand {
    fn into(self) -> Message {
        self.to_string().into()
    }
}
