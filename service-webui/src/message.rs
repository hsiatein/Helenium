use heleny_proto::resource::Resource;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub enum SessionMessage {
    Register {
        sender: mpsc::Sender<ServiceMessage>,
        feedback: oneshot::Sender<()>,
    },
    UserInput {
        input: String,
    },
    Logout,
}

#[derive(Debug, Clone, Serialize)]
pub enum ServiceMessage {
    UpdateResource(Resource),
}

#[derive(Debug)]
pub struct SessionToService {
    pub token: Uuid,
    pub payload: SessionMessage,
}
