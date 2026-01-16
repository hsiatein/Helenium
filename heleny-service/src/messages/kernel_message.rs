use tokio::sync::mpsc;

#[derive(Debug)]
pub enum KernelMessage {
    Shutdown,
    GetBusStatsRx {
        sender: mpsc::Sender<(String, String)>,
    },
    SetUser {
        name: String,
    },
}
