use crate::WebuiService;
use anyhow::Result;
use heleny_proto::DISPLAY_MESSAGES;
use heleny_proto::FrontendMessage;
use heleny_proto::MEMORY_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_service::MemoryServiceMessage;
use tokio::sync::oneshot;
use uuid::Uuid;

impl WebuiService {
    pub async fn handle_get_history(&mut self, session: Uuid, id_upper_bound: i64) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.endpoint
            .send(
                MEMORY_SERVICE,
                MemoryServiceMessage::Get {
                    id_upper_bound,
                    feedback: tx,
                },
            )
            .await?;
        let history = rx.await?;
        self.send_to_session(
            session,
            FrontendMessage::UpdateResource(Resource {
                name: DISPLAY_MESSAGES.to_string(),
                payload: ResourcePayload::DisplayMessages {
                    new: false,
                    messages: history,
                },
            }),
        )
        .await
    }
}
