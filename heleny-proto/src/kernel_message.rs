use tokio::sync::oneshot;

use crate::health::KernelHealth;



#[derive(Debug)]
pub enum KernelMessage {
    Shutdown,
    GetHealth(oneshot::Sender<KernelHealth>),
    UploadStatus(ServiceStatus),
}

#[derive(Debug)]
pub enum ServiceStatus {
    Ready,
    Alive,
    InitFail,
}
