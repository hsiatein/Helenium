use heleny_proto::ResourcePayload;
use tokio::sync::oneshot;
use tokio::sync::watch;

#[derive(Debug)]
pub enum HubServiceMessage {
    Publish {
        resource_name: String,
        receiver: watch::Receiver<ResourcePayload>,
    },
    Unpublish {
        resource_name: String,
    },
    Subscribe {
        resource_name: String,
    },
    Unsubscribe {
        resource_name: String,
    },
    Get {
        resource_name: String,
        feedback: oneshot::Sender<ResourcePayload>,
    },
}
