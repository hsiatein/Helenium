use serde::Deserialize;
use serde::Serialize;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub struct ConsentRequestion {
    pub task_id: Uuid,
    pub task_description: String,
    pub reason: String,
    pub description: String,
    pub feedback: oneshot::Sender<bool>,
}

impl ConsentRequestion {
    pub fn to_frontend(&self, request_id: Uuid) -> ConsentRequestionFE {
        let requestion_fe = ConsentRequestionFE {
            request_id,
            task_id: self.task_id,
            task_description: self.task_description.clone(),
            reason: self.reason.clone(),
            descripion: self.description.clone(),
        };
        requestion_fe
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequestionFE {
    pub request_id: Uuid,
    pub task_id: Uuid,
    pub task_description: String,
    pub reason: String,
    pub descripion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserDecision {
    ConsentRequestions(Vec<ConsentRequestionFE>),
}
