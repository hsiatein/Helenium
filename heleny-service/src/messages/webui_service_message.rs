use heleny_proto::UserDecision;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebuiServiceMessage {
    UserDecision(UserDecision)
}
