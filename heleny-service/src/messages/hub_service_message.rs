use heleny_proto::resource::{ResourcePayload};
use tokio::sync::watch;

#[derive(Debug)]
pub enum HubServiceMessage {
    Publish {
        provider: String,
        resource_name: String,
        receiver: watch::Receiver<ResourcePayload>,
    },
    Subscribe {
        resource_name: String,
    },
    Unsubscribe {
        resource_name: String,
    },
}
