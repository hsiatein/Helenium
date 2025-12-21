use tokio::sync::oneshot;

pub enum CommonMessage {
    Stop(oneshot::Sender<()>),
}
