use heleny_proto::ChatRole;
use heleny_proto::MemoryEntry;
use heleny_proto::MemoryContent;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum MemoryServiceMessage {
    Post {
        role: ChatRole,
        content: MemoryContent
    },
    Get {
        id_upper_bound: i64,
        feedback: oneshot::Sender<Vec<MemoryEntry>>,
    },
    GetMemoryEntries {
        feedback: oneshot::Sender<Vec<MemoryEntry>>,
    },
    GetSimilarMemoryEntries {
        content: String,
        num: usize,
        feedback: oneshot::Sender<Vec<MemoryEntry>>,
    },
    Delete {
        id: i64
    },
    SetEmbedAvailable {
        available:bool
    }
}
