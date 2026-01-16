use crate::WebuiService;
use anyhow::Result;
use heleny_proto::FrontendMessage;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::TASK_SERVICE;
use heleny_proto::WEBUI_SERVICE;
use heleny_service::TaskServiceMessage;
use heleny_service::WebuiServiceMessage;
use uuid::Uuid;

impl WebuiService {
    pub async fn handle_toggle_task_logs(
        &mut self,
        session: Uuid,
        id: Uuid,
        expanded: bool,
    ) -> Result<()> {
        let session_task_logs = match self.session_task_logs.get_mut(&session) {
            Some(logs) => logs,
            None => return Ok(()),
        };
        if expanded {
            let endpoint = self.endpoint.create_sender_endpoint();
            let handle = tokio::spawn(async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel(32);
                let _ = endpoint
                    .send(
                        TASK_SERVICE,
                        TaskServiceMessage::SubscribeTaskLogs { id, sender: tx },
                    )
                    .await;
                while let Some(log) = rx.recv().await {
                    let _ = endpoint
                        .send(
                            WEBUI_SERVICE,
                            WebuiServiceMessage::SendToFrontend {
                                session,
                                message: FrontendMessage::UpdateResource(Resource {
                                    name: String::new(),
                                    payload: ResourcePayload::TaskLogs { id, logs: log.log },
                                }),
                            },
                        )
                        .await;
                }
            });
            session_task_logs.insert(id, handle);
        } else {
            if let Some(handle) = session_task_logs.remove(&id) {
                handle.abort();
            }
        }
        Ok(())
    }
}
