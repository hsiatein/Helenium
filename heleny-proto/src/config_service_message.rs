use tokio::sync::oneshot;
use serde_json::Value;

#[derive(Debug)]
pub enum ConfigServiceMessage {
    Get {
        sender: oneshot::Sender<Option<Value>>,
    },
    Set {
        value: Value,
    },
    Update,
    Persist,
}
