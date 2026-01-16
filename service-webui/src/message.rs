use heleny_proto::FrontendMessage;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub enum SessionMessage {
    Register {
        sender: mpsc::Sender<FrontendMessage>,
        feedback: oneshot::Sender<()>,
    },
    UserInput {
        input: String,
    },
    Logout,
}

#[derive(Debug)]
pub struct SessionToService {
    pub token: Uuid,
    pub payload: SessionMessage,
}
