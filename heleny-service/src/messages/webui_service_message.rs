use heleny_proto::FrontendMessage;
use heleny_proto::UserDecision;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebuiServiceMessage {
    UserDecision(UserDecision),
    SendToFrontend {
        session: Uuid,
        message: FrontendMessage,
    },
}
