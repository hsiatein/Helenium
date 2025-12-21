use tokio::sync::oneshot;

use crate::health::KernelHealth;

pub enum KernelMessage {
    Shutdown,
    GetHealth(oneshot::Sender<KernelHealth>),
}
