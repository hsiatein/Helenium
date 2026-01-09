use crate::WebuiService;
use anyhow::Result;
use base64::prelude::*;
use heleny_proto::CHAT_SERVICE;
use heleny_proto::DISPLAY_MESSAGES;
use heleny_proto::FS_SERVICE;
use heleny_proto::FrontendCommand;
use heleny_proto::FrontendMessage;
use heleny_proto::HEALTH;
use heleny_proto::HUB_SERVICE;
use heleny_proto::KERNEL_NAME;
use heleny_proto::MEMORY_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::USER_SERVICE;
use heleny_proto::UserDecision;
use heleny_proto::WEBUI_SERVICE;
use heleny_service::ChatServiceMessage;
use heleny_service::FsServiceMessage;
use heleny_service::HubServiceMessage;
use heleny_service::KernelMessage;
use heleny_service::MemoryServiceMessage;
use heleny_service::UserServiceMessage;
use heleny_service::WebuiServiceMessage;
use tokio::sync::oneshot;
use uuid::Uuid;

impl WebuiService {
    pub async fn handle_command(&mut self, session: Uuid, command: FrontendCommand) -> Result<()> {
        match command {
            FrontendCommand::UserInput(input)=>{
                self.endpoint
                    .send(CHAT_SERVICE, ChatServiceMessage::Chat { message: input })
                    .await
            }
            FrontendCommand::GetHistory(id_upper_bound) => {
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
            FrontendCommand::GetHealth => {
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
                    session,
                    FrontendMessage::UpdateResource(Resource {
                        name: HEALTH.to_string(),
                        payload: health,
                    }),
                )
                .await
            }
            FrontendCommand::Shutdown => {
                self.endpoint
                    .send(KERNEL_NAME, KernelMessage::Shutdown)
                    .await
            }
            FrontendCommand::GetImage { id, path } => {
                let (tx, rx) = oneshot::channel();
                self.endpoint
                    .send(
                        FS_SERVICE,
                        FsServiceMessage::GetImage { path, feedback: tx },
                    )
                    .await?;
                let sub = self.endpoint.create_sender_endpoint();
                tokio::spawn(async move {
                    let Ok(image) = rx.await else {
                        return;
                    };
                    let base64 = BASE64_STANDARD.encode(image);
                    let _ = sub
                        .send(
                            WEBUI_SERVICE,
                            WebuiServiceMessage::SendToFrontend {
                                session,
                                message: FrontendMessage::UpdateResource(Resource {
                                    name: String::new(),
                                    payload: ResourcePayload::Image { id, base64 },
                                }),
                            },
                        )
                        .await;
                });
                Ok(())
            }
            FrontendCommand::GetConsentRequestions => {
                let (tx, rx) = oneshot::channel();
                self.endpoint
                    .send(
                        USER_SERVICE,
                        UserServiceMessage::ListConsentRequestions { feedback: tx },
                    )
                    .await?;
                let result = rx.await?;
                let user_decision = UserDecision::ConsentRequestions(result);
                self.send_to_session(session, FrontendMessage::UserDecision(user_decision))
                    .await
            }
            FrontendCommand::MakeDecision { req_id, approval } => {
                self.endpoint
                    .send(
                        USER_SERVICE,
                        UserServiceMessage::MakeDecision { req_id, approval },
                    )
                    .await
            }
            FrontendCommand::Close=>{
                Ok(())
            }
        }
    }
}
