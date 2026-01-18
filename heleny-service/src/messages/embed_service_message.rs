use std::collections::{HashMap, HashSet};

use tokio::sync::oneshot;

#[derive(Debug)]
pub enum EmbedServiceMessage {
    Embed {
        id: i64,
        content: String,
    },
    EmbedBatch {
        batch:HashMap<i64,String>
    },
    Delete {
        id: i64
    },
    Search {
        content: String,
        num: usize,
        feedback: oneshot::Sender<HashSet<i64>>
    },
    GetAllID {
        feedback: oneshot::Sender<HashSet<i64>>
    }
}