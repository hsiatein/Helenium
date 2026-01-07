use anyhow::Result;
use heleny_proto::DISPLAY_MESSAGES;
use heleny_proto::FrontendCommand;
use heleny_proto::FrontendMessage;
use heleny_proto::HEALTH;
use heleny_proto::HUB_SERVICE;
use heleny_proto::KERNEL_NAME;
use heleny_proto::MEMORY_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_service::HubServiceMessage;
use heleny_service::KernelMessage;
use heleny_service::MemoryServiceMessage;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::WebuiService;

impl WebuiService {
    pub async fn handle_command(&mut self, token: Uuid, command: FrontendCommand) -> Result<()> {
        match command {
            FrontendCommand::GetHistory(id_upper_bound)=>{
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
                    token,
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
            FrontendCommand::GetHealth=>{
                let (tx, rx) = oneshot::channel();
                self.endpoint
                    .send(
                        HUB_SERVICE,
                        HubServiceMessage::Get {
                            resource_name: HEALTH.to_string(),
                            feedback: tx,
                        },
                    )
                    .await?;
                let health = rx.await?;
                self.send_to_session(
                    token,
                    FrontendMessage::UpdateResource(Resource {
                        name: HEALTH.to_string(),
                        payload: health,
                    }),
                )
                .await
            }
            FrontendCommand::Shutdown=>{
                self.endpoint
                    .send(KERNEL_NAME, KernelMessage::Shutdown)
                    .await
            }
        }
    }
}
