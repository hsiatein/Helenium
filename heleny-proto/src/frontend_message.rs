use serde::{Deserialize, Serialize};
use crate::resource::Resource;

#[derive(Debug, Clone, Serialize,Deserialize)]
pub enum FrontendMessage {
    UpdateResource(Resource),
}