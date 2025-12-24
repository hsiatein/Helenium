use tokio::sync::oneshot;

/// path的各个键用.分割, 如data["a"]["b"]["c"] => path="a.b.c"
#[derive(Debug)]
pub enum ConfigServiceMessage {
    Get {
        sender: oneshot::Sender<Option<toml::Value>>,
    },
    Set {
        value: toml::Value,
    },
    Update,
    Persist,
}
