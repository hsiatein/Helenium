use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::CanRequestConsent;
use heleny_proto::HelenyTool;
use heleny_proto::HelenyToolFactory;
use heleny_proto::SCHEDULE_SERVICE;
use heleny_proto::ScheduledTask;
use heleny_proto::get_tool_arg;
use heleny_service::ScheduleServiceMessage;
use serde_json::Value;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub struct ScheduleToolFactory {
    endpoint: Endpoint,
    offset: i32,
}

impl ScheduleToolFactory {
    pub fn new(endpoint: Endpoint, offset: i32) -> Self {
        ScheduleToolFactory { endpoint, offset }
    }
}

#[async_trait]
impl HelenyToolFactory for ScheduleToolFactory {
    fn name(&self) -> String {
        "schedule".to_string()
    }
    async fn create(&mut self) -> Result<Box<dyn HelenyTool>> {
        let tool = ScheduleTool::new(self.endpoint.create_sender_endpoint(), self.offset);
        Ok(Box::new(tool))
    }
}

#[derive(Debug)]
pub struct ScheduleTool {
    endpoint: Endpoint,
    offset: i32,
}

impl ScheduleTool {
    pub fn new(endpoint: Endpoint, offset: i32) -> Self {
        Self { endpoint, offset }
    }
}

#[async_trait]
impl HelenyTool for ScheduleTool {
    async fn invoke(
        &mut self,
        command: String,
        mut args: HashMap<String, Value>,
        request: Box<&dyn CanRequestConsent>,
    ) -> Result<String> {
        match command.as_str() {
            "once" => {
                let time: String = get_tool_arg(&mut args, "time")?;
                let description: String = get_tool_arg(&mut args, "description")?;
                let task = ScheduledTask::from_once(description, self.offset, &time)?;
                request
                    .request_consent(format!("申请新建日程任务: {:?}", task))
                    .await?;
                self.endpoint
                    .send(SCHEDULE_SERVICE, ScheduleServiceMessage::AddTask { task })
                    .await?;
                Ok("新建 Once 日程任务完成".into())
            }
            "interval" => {
                let every: u64 = get_tool_arg(&mut args, "every")?;
                let description: String = get_tool_arg(&mut args, "description")?;
                let task = ScheduledTask::from_interval(description, self.offset, every)?;
                request
                    .request_consent(format!("申请新建日程任务: {:?}", task))
                    .await?;
                self.endpoint
                    .send(SCHEDULE_SERVICE, ScheduleServiceMessage::AddTask { task })
                    .await?;
                Ok("新建 Interval 日程任务完成".into())
            }
            "cron" => {
                let cron: String = get_tool_arg(&mut args, "cron")?;
                let description: String = get_tool_arg(&mut args, "description")?;
                let task = ScheduledTask::from_cron(description, self.offset, &cron)?;
                request
                    .request_consent(format!("申请新建日程任务: {:?}", task))
                    .await?;
                self.endpoint
                    .send(SCHEDULE_SERVICE, ScheduleServiceMessage::AddTask { task })
                    .await?;
                Ok("新建 Cron 日程任务完成".into())
            }
            "list" => {
                let (tx, rx) = oneshot::channel();
                self.endpoint
                    .send(
                        SCHEDULE_SERVICE,
                        ScheduleServiceMessage::ListTask { feedback: tx },
                    )
                    .await?;
                let list = rx.await.context("获取日程任务列表失败")?;
                Ok(format!("{:?}", list))
            }
            "cancel" => {
                let id: Uuid = get_tool_arg(&mut args, "id")?;
                request
                    .request_consent(format!("申请取消日程任务: {}", id))
                    .await?;
                self.endpoint
                    .send(SCHEDULE_SERVICE, ScheduleServiceMessage::CancelTask { id })
                    .await?;
                Ok("取消日程任务完成".into())
            }
            cmd => Err(anyhow::anyhow!("未知命令: {}", cmd)),
        }
    }
}
