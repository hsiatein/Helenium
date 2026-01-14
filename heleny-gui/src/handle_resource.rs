use crate::FrontendHandler;
use anyhow::Result;
use heleny_proto::ResourcePayload;
use tracing::debug;

mod handle_display_messages;
mod handle_image;
mod handle_slint_health;
mod handle_slint_schedules;
mod handle_task_abstract;
mod handle_task_logs;
mod handle_tool_abstracts;
mod handle_total_bus_traffic;

impl FrontendHandler {
    pub async fn handle_resource(&self, resource: ResourcePayload) -> Result<()> {
        match resource {
            ResourcePayload::TotalBusTraffic(data) => self.handle_total_bus_traffic(data).await,
            ResourcePayload::DisplayMessages { new, messages } => {
                debug!("{:?}", messages);
                self.handle_display_messages(new, messages).await
            }
            ResourcePayload::Health(health) => {
                debug!("{:?}", health);
                self.handle_health(health).await
            }
            ResourcePayload::Image { id, base64 } => self.handle_image(id, base64).await,
            ResourcePayload::TaskAbstract { task_abstracts } => {
                debug!("任务摘要: {:?}", task_abstracts);
                self.handle_task_abstract(task_abstracts).await
            }
            ResourcePayload::TaskLogs { id, logs } => self.handle_task_logs(id, logs).await,
            ResourcePayload::Schedules { schedules } => {
                debug!("Schedule: {:?}", schedules);
                self.handle_schedules(schedules).await
            }
            ResourcePayload::ToolAbstracts { abstracts } => {
                debug!("ToolAbstracts: {:?}", abstracts);
                self.handle_tool_abstracts(abstracts).await
            }
        }
    }
}
