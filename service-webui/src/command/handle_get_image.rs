use crate::WebuiService;
use anyhow::Result;
use base64::prelude::*;
use heleny_proto::FS_SERVICE;
use heleny_proto::FrontendMessage;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::WEBUI_SERVICE;
use heleny_service::FsServiceMessage;
use heleny_service::WebuiServiceMessage;
use std::path::PathBuf;
use tokio::sync::oneshot;
use uuid::Uuid;

impl WebuiService {
    pub async fn handle_get_image(&mut self, session: Uuid, id: i64, path: PathBuf) -> Result<()> {
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
}
