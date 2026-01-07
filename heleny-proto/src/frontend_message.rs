use crate::UserDecision;
use crate::resource::Resource;
use serde::Deserialize;
use serde::Serialize;
use tungstenite::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrontendMessage {
    UpdateResource(Resource),
    UserDecision(UserDecision)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrontendCommand {
    GetHistory(i64),
    GetHealth,
    Shutdown,
}

impl FrontendCommand {
    pub fn to_string(&self) -> String {
        "!".to_string()+&serde_json::to_string(self).unwrap_or(String::new())
    }
}

impl Into<Message> for FrontendCommand {
    fn into(self) -> Message {
        self.to_string().into()
    }
}