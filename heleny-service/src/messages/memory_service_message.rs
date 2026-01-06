use async_openai::types::chat::ChatCompletionRequestMessage;
use heleny_proto::DisplayMessage;
use heleny_proto::MemoryEntry;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum MemoryServiceMessage {
    Post {
        entry: MemoryEntry,
    },
    Get {
        id_upper_bound: i64,
        feedback: oneshot::Sender<Vec<DisplayMessage>>,
    },
    GetChatMemories {
        feedback: oneshot::Sender<Vec<ChatCompletionRequestMessage>>,
    },
}
