use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use tokio::sync::oneshot;

use heleny_proto::KernelHealth;
use heleny_proto::ServiceHandle;

#[derive(Debug)]
pub enum ServiceSignal {
    Ready,
    Alive,
    InitFail,
    Terminate(String),
}

#[derive(Debug)]
pub enum KernelServiceMessage {
    // System
    StopAll(oneshot::Sender<()>),
    Init,
    InitParams(
        Arc<Mutex<KernelHealth>>,
        Arc<Mutex<HashMap<String, ServiceHandle>>>,
    ),
    // Standard
    GetHealth(oneshot::Sender<KernelHealth>),
    UploadStatus(ServiceSignal),
    WaitFor {
        name: String,
        sender: oneshot::Sender<Result<()>>,
    },
}
