use crate::UserDecision;
use crate::resource::Resource;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrontendMessage {
    UpdateResource(Resource),
    UserDecision(UserDecision)
}
