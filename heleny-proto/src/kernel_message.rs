use tokio::sync::oneshot;

use crate::health::KernelHealth;

#[derive(Debug)]
pub enum KernelMessage {
    Shutdown,
    GetHealth(oneshot::Sender<KernelHealth>),
    Alive,
}
