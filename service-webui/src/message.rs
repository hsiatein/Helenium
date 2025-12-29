use heleny_proto::resource::Resource;
use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;


#[derive(Debug)]
pub enum SessionMessage{
    Register{
        sender:mpsc::Sender<ServiceMessage>,
        feedback:oneshot::Sender<()>,
    }
}

#[derive(Debug,Clone,Serialize)]
pub enum ServiceMessage{
    UpdateResource(Resource)
}

#[derive(Debug)]
pub struct SessionToService {
    pub token:Uuid,
    pub payload:SessionMessage,
}