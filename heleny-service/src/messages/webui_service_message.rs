use heleny_proto::{FrontendMessage, UserDecision};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebuiServiceMessage {
    UserDecision(UserDecision),
    SendToFrontend {
        session: Uuid,
        message: FrontendMessage
    }
}
