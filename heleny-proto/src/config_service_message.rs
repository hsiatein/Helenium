use tokio::sync::oneshot;

pub enum ConfigServiceMessage {
    Get {
        key: &'static str,
        sender: oneshot::Sender<toml::Value>,
    },
    Set {
        key: &'static str,
        value: toml::Value,
    },
    Update,
    Persist,
}
