use std::collections::VecDeque;

use tokio::sync::oneshot;

#[derive(Debug)]
pub enum StatsServiceMessage {
    GetBusStats {
        sender: oneshot::Sender<VecDeque<usize>>,
    },
}
