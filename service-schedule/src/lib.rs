use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use chrono::FixedOffset;
use chrono::Utc;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::FS_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::SCHEDULE;
use heleny_proto::ScheduledTask;
use heleny_proto::ServiceRole;
use heleny_proto::TASK_SERVICE;
use heleny_proto::downcast;
use heleny_service::FsServiceMessage;
use heleny_service::ScheduleServiceMessage;
use heleny_service::Service;
use heleny_service::TaskServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::publish_resource;
use heleny_service::read_via_fs_service;
use heleny_service::register_tool_factory;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

use crate::config::ScheduleConfig;
use crate::tool::ScheduleToolFactory;

mod config;
mod tool;

#[base_service(deps=["ConfigService","FsService","TaskService","ToolkitService","HubService"])]
pub struct ScheduleService {
    endpoint: Endpoint,
    offset: FixedOffset,
    scheduled_tasks: HashMap<Uuid, ScheduledTask>,
    notifier: Option<JoinHandle<()>>,
    schedule_path: PathBuf,
    schedule_tx: watch::Sender<ResourcePayload>,
}

#[derive(Debug)]
enum WorkerMessage {
    IsReady,
}

#[async_trait]
impl Service for ScheduleService {
    type MessageType = ScheduleServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config: ScheduleConfig = get_from_config_service(&endpoint).await?;
        let schedule_path = PathBuf::from(config.schedule_dir).join("schedule.json");
        let offset = FixedOffset::east_opt(config.offset).context("获取 offset 失败")?;
        let schedule_str = read_via_fs_service(&endpoint, &schedule_path).await;
        let mut schedule: HashMap<Uuid, ScheduledTask> = match schedule_str {
            Ok(str) => serde_json::from_str(&str)?,
            Err(_) => HashMap::new(),
        };
        schedule
            .values_mut()
            .for_each(|task| task.update_next_trigger());
        // 向工具服务注册
        let factory = ScheduleToolFactory::new(endpoint.create_sender_endpoint(), config.offset);
        register_tool_factory(&endpoint, factory).await;
        // 向 Hub 服务注册
        let (tx, rx) = watch::channel(ResourcePayload::Schedules {
            schedules: schedule.clone(),
        });
        publish_resource(&endpoint, SCHEDULE, rx).await?;
        // 实例化
        let mut instance = Self {
            endpoint,
            offset,
            scheduled_tasks: schedule,
            notifier: None,
            schedule_path,
            schedule_tx: tx,
        };
        instance.find_next_trigger();
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: ScheduleServiceMessage,
    ) -> Result<()> {
        match msg {
            ScheduleServiceMessage::AddTask { mut task } => {
                task.update_next_trigger();
                self.scheduled_tasks.insert(Uuid::new_v4(), task);
                self.find_next_trigger();
                self.persist().await
            }
            ScheduleServiceMessage::ListTask { feedback } => {
                let _ = feedback.send(self.scheduled_tasks.clone());
                Ok(())
            }
            ScheduleServiceMessage::CancelTask { id } => {
                let elem = self.scheduled_tasks.remove(&id);
                if elem.is_some() {
                    self.find_next_trigger();
                    self.persist().await
                } else {
                    Ok(())
                }
            }
            ScheduleServiceMessage::Reload=>{
                let schedule_str = read_via_fs_service(&self.endpoint, &self.schedule_path).await?;
                let mut schedule: HashMap<Uuid, ScheduledTask> = serde_json::from_str(&schedule_str)?;
                schedule
                    .values_mut()
                    .for_each(|task| task.update_next_trigger());
                self.scheduled_tasks=schedule;
                self.find_next_trigger();
                self.push_schedule_resource();
                Ok(())
            }
        }
    }
    async fn stop(&mut self) {
        if let Some(notifier) = self.notifier.take() {
            notifier.abort();
        }
        let _ = self.persist().await;
    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()> {
        let msg: WorkerMessage = downcast(msg)?;
        match msg {
            WorkerMessage::IsReady => {
                for (_, task) in &mut self.scheduled_tasks {
                    if task.is_ready() {
                        let _ = self
                            .endpoint
                            .send(
                                TASK_SERVICE,
                                TaskServiceMessage::AddTask {
                                    task_description: task.description.clone(),
                                },
                            )
                            .await;
                        task.update_next_trigger();
                    }
                }
                self.find_next_trigger();
                self.persist().await
            }
        }
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl ScheduleService {
    /// 持久化 schedule
    async fn persist(&self) -> Result<()> {
        self.push_schedule_resource();
        let content = serde_json::to_string(&self.scheduled_tasks)?;
        let (tx, rx) = oneshot::channel();
        self.endpoint
            .send(
                FS_SERVICE,
                FsServiceMessage::Write {
                    path: self.schedule_path.clone(),
                    content,
                    feedback: tx,
                },
            )
            .await?;
        rx.await.context("写入 schedule.json 失败")?;
        Ok(())
    }
    fn push_schedule_resource(&self){
        if let Err(e) = self.schedule_tx.send(ResourcePayload::Schedules {
            schedules: self.scheduled_tasks.clone(),
        }) {
            warn!("推送 Schedule 资源失败: {}", e);
        };
    }
    /// 不会更新每个任务的下次时间，清除给不出下次时间的任务，开一个任务提醒服务
    fn find_next_trigger(&mut self) {
        let mut next = None;
        self.scheduled_tasks
            .retain(|id, task| match task.next_trigger {
                Some(trigger) => {
                    let Some(next) = &mut next else {
                        next = Some(trigger);
                        return true;
                    };
                    if *next > trigger {
                        *next = trigger;
                    };
                    true
                }
                None => {
                    info!("任务 {} 没有下次触发，删除: {}", id, task.description);
                    false
                }
            });
        let Some(next) = next else {
            return;
        };
        let offset = self.offset;
        let endpoint = self
            .endpoint
            .create_sub_endpoint()
            .expect("服务创建sub endpoint必须成功");
        let handle = tokio::spawn(async move {
            let millis = (next - Utc::now().with_timezone(&offset))
                .num_milliseconds()
                .max(0) as u64;
            tokio::time::sleep(Duration::from_millis(millis)).await;
            let _ = endpoint.send(Box::new(WorkerMessage::IsReady)).await;
        });
        if let Some(handle) = &mut self.notifier {
            handle.abort();
        }
        self.notifier = Some(handle);
    }
}
