use crate::WebuiService;
use anyhow::Result;
use heleny_proto::CHAT_SERVICE;
use heleny_proto::FrontendCommand;
use heleny_proto::FrontendMessage;
use heleny_proto::HEALTH;
use heleny_proto::KERNEL_NAME;
use heleny_proto::MCP_SERVICE;
use heleny_proto::Resource;
use heleny_proto::SCHEDULE;
use heleny_proto::SCHEDULE_SERVICE;
use heleny_proto::TASK_SERVICE;
use heleny_proto::TOOL_ABSTRACTS;
use heleny_proto::TOOLKIT_SERVICE;
use heleny_proto::USER_SERVICE;
use heleny_proto::UserDecision;
use heleny_service::ChatServiceMessage;
use heleny_service::KernelMessage;
use heleny_service::McpServiceMessage;
use heleny_service::ScheduleServiceMessage;
use heleny_service::TaskServiceMessage;
use heleny_service::ToolkitServiceMessage;
use heleny_service::UserServiceMessage;
use heleny_service::get_resource;
use tokio::sync::oneshot;
use uuid::Uuid;

mod handle_get_history;
mod handle_get_image;
mod handle_toggle_task_logs;

impl WebuiService {
    pub async fn handle_command(&mut self, session: Uuid, command: FrontendCommand) -> Result<()> {
        match command {
            FrontendCommand::UserInput(input) => {
                self.endpoint
                    .send(CHAT_SERVICE, ChatServiceMessage::Chat { message: input })
                    .await
            }
            FrontendCommand::GetHistory(id_upper_bound) => {
                self.handle_get_history(session, id_upper_bound).await
            }
            FrontendCommand::GetHealth => {
                let health = get_resource(&self.endpoint, HEALTH).await?;
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
                self.handle_get_image(session, id, path).await
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
            FrontendCommand::Close => Ok(()),
            FrontendCommand::CancelTask { id } => {
                self.endpoint
                    .send(TASK_SERVICE, TaskServiceMessage::CancelTask { id })
                    .await
            }
            FrontendCommand::ToggleTaskLogs { id, expanded } => {
                self.handle_toggle_task_logs(session, id, expanded).await
            }
            FrontendCommand::GetSchedules => {
                let resource = get_resource(&self.endpoint, SCHEDULE).await?;
                self.send_to_session(
                    session,
                    FrontendMessage::UpdateResource(Resource {
                        name: "".into(),
                        payload: resource,
                    }),
                )
                .await
            }
            FrontendCommand::CancelSchedule { id } => {
                self.endpoint
                    .send(SCHEDULE_SERVICE, ScheduleServiceMessage::CancelTask { id })
                    .await
            }
            FrontendCommand::GetToolAbstrats => {
                let resource = get_resource(&self.endpoint, TOOL_ABSTRACTS).await?;
                self.send_to_session(
                    session,
                    FrontendMessage::UpdateResource(Resource {
                        name: "".into(),
                        payload: resource,
                    }),
                )
                .await
            }
            FrontendCommand::ReloadTools => {
                self.endpoint
                    .send(TOOLKIT_SERVICE, ToolkitServiceMessage::Reload)
                    .await?;
                self.endpoint
                    .send(MCP_SERVICE, McpServiceMessage::Reload)
                    .await
            }
        }
    }
}
