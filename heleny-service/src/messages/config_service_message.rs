use serde_json::Value;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ConfigServiceMessage {
    Get {
        sender: oneshot::Sender<Option<Value>>,
    },
    Set {
        value: Value,
    },
    Update{
        feedback: oneshot::Sender<()>,
    },
    Persist,
    Export {
        key: String,
        value: Value,
    },
    Import {
        key: String,
        feedback: oneshot::Sender<Value>,
    },
}
