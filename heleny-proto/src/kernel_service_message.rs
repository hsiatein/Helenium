use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::oneshot;

use crate::{health::KernelHealth, service_handle::ServiceHandle};

#[derive(Debug)]
pub enum ServiceSignal {
    Ready,
    Alive,
    InitFail,
    Terminate,
}

#[derive(Debug)]
pub enum KernelServiceMessage {
    // System
    StopAll(oneshot::Sender<()>),
    Init,
    InitParams(
        Arc<Mutex<KernelHealth>>,
        Arc<Mutex<HashMap<&'static str, ServiceHandle>>>,
    ),
    // Standard
    GetHealth(oneshot::Sender<KernelHealth>),
    UploadStatus(ServiceSignal),
}
