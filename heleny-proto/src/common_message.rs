use tokio::sync::oneshot;

#[derive(Debug)]
pub enum CommonMessage {
    Stop(oneshot::Sender<()>),
}
