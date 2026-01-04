use heleny_proto::resource::ResourcePayload;
use tokio::sync::{oneshot, watch};

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
    Get {
        resource_name: String,
        feedback:oneshot::Sender<ResourcePayload>,
    }
}
