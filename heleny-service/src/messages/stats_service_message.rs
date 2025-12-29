use std::collections::VecDeque;

use chrono::{DateTime, Local};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum StatsServiceMessage {
    GetBusStats {
        sender: oneshot::Sender<VecDeque<(DateTime<Local>,usize)>>,
    },
}
