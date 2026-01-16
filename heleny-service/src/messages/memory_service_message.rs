use async_openai::types::chat::ChatCompletionRequestMessage;
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
    GetChatMemories {
        feedback: oneshot::Sender<Vec<ChatCompletionRequestMessage>>,
    },
    Delete {
        id: i64
    }
}
