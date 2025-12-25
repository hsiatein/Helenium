use std::collections::HashMap;

use tokio::sync::mpsc;

#[derive(Debug)]
pub enum KernelMessage {
    Shutdown,
    GetBusStatsRx {
        sender: mpsc::Sender<HashMap<&'static str, usize>>,
    },
}
