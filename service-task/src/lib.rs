use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::CHAT_SERVICE;
use heleny_proto::ExecutorModel;
use heleny_proto::PlannerModel;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::TASK_ABSTRACT;
use heleny_proto::TOOLKIT_SERVICE;
use heleny_proto::TaskStatus;
use heleny_proto::downcast;
use heleny_service::ChatServiceMessage;
use heleny_service::Service;
use heleny_service::TaskServiceMessage;
use heleny_service::Toolkit;
use heleny_service::ToolkitServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::publish_resource;
use std::collections::HashMap;
use std::collections::VecDeque;
use tokio::sync::oneshot;
use tokio::time::Instant;
use tracing::warn;

mod task;
pub use task::*;
mod config;
pub use config::*;
use tracing::info;
use uuid::Uuid;
mod task_logger;
pub use task_logger::*;

#[base_service(deps=["ConfigService","ChatService","HubService"])]
pub struct TaskService {
    endpoint: Endpoint,
    running_tasks: HashMap<Uuid, TaskHandle>,
    pending_tasks: VecDeque<Task>,
    task_logs: TaskLoggerHandle,
    config: TaskConfig,
}

#[derive(Debug)]
enum WorkerMessage {
    Finish {
        id: Uuid,
        success: bool,
    },
    GetPlanner {
        feedback: oneshot::Sender<PlannerModel>,
    },
    GetExecutor {
        feedback: oneshot::Sender<ExecutorModel>,
    },
    GetToolkit {
        tool_names: Vec<String>,
        task_id: Uuid,
        task_description: String,
        feedback: oneshot::Sender<Toolkit>,
    },
}

#[async_trait]
impl Service for TaskService {
    type MessageType = TaskServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config: TaskConfig = get_from_config_service(&endpoint).await?;
        let (task_logs, watch_rx) = launch_task_logger().await;
        publish_resource(&endpoint, TASK_ABSTRACT, watch_rx).await?;
        let instance = Self {
            endpoint,
            running_tasks: HashMap::new(),
            pending_tasks: VecDeque::new(),
            task_logs,
            config,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: TaskServiceMessage,
    ) -> Result<()> {
        match msg {
            TaskServiceMessage::AddTask { task_description } => {
                let task = Task::new(
                    Uuid::new_v4(),
                    task_description,
                    self.endpoint.create_sub_endpoint()?,
                    self.task_logs.get_log_sender(),
                    self.config.max_working_loop,
                );
                let _ = self
                    .task_logs
                    .add_task(task.id, task.task_description.clone())
                    .await;
                info!("已添加新任务 {} : {}", task.id, task.task_description);
                self.pending_tasks.push_back(task);
                self.launch_tasks().await;
            }
            TaskServiceMessage::CancelTask { id } => {
                if let Some(handle) = self.running_tasks.remove(&id) {
                    handle.handle.abort();
                    let _ = self.task_logs.set_status(id, TaskStatus::Canceled).await;
                    self.launch_tasks().await;
                } else if self
                    .pending_tasks
                    .iter()
                    .find(|task| task.id == id)
                    .is_some()
                {
                    self.pending_tasks.retain(|task| task.id != id);
                    self.task_logs.set_status(id, TaskStatus::Canceled).await?;
                }
            }
            TaskServiceMessage::SubscribeTaskLogs { id, sender } => {
                self.task_logs.subscribe(id, sender).await?;
            }
        }
        Ok(())
    }
    async fn stop(&mut self) {
        if let Err(e) = self.task_logs.stop().await {
            warn!("停止 TaskLogger 失败: {}", e);
        }
    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()> {
        let msg: WorkerMessage = downcast(msg)?;
        match msg {
            WorkerMessage::Finish { id, success } => {
                let handle = self.running_tasks.remove(&id).context("没有此 ID 的任务")?;
                handle.handle.abort();
                self.launch_tasks().await;
                let log = self.task_logs.get_log(id).await?;
                if success {
                    let _ = self.task_logs.set_status(id, TaskStatus::Success).await;
                    info!("任务 {} 成功: {:?}", id, log);
                } else {
                    let _ = self.task_logs.set_status(id, TaskStatus::Fail).await;
                    info!("任务 {} 失败: {:?}", id, log);
                }
                self.endpoint
                    .send(
                        CHAT_SERVICE,
                        ChatServiceMessage::TaskFinished { log: log.get_log() },
                    )
                    .await
            }
            WorkerMessage::GetPlanner { feedback } => {
                self.endpoint
                    .send(CHAT_SERVICE, ChatServiceMessage::GetPlanner { feedback })
                    .await
            }
            WorkerMessage::GetExecutor { feedback } => {
                self.endpoint
                    .send(CHAT_SERVICE, ChatServiceMessage::GetExecutor { feedback })
                    .await
            }
            WorkerMessage::GetToolkit {
                tool_names,
                task_id,
                task_description,
                feedback,
            } => {
                self.endpoint
                    .send(
                        TOOLKIT_SERVICE,
                        ToolkitServiceMessage::GetToolkit {
                            tool_names,
                            task_id,
                            task_description,
                            feedback,
                        },
                    )
                    .await
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

impl TaskService {
    async fn launch_tasks(&mut self) {
        while self.running_tasks.len() < self.config.max_running_tasks {
            let Some(task) = self.pending_tasks.pop_front() else {
                return;
            };
            let _ = self
                .task_logs
                .set_status(task.id, TaskStatus::Running)
                .await;
            info!("已启动新任务 {} : {}", task.id, task.task_description);
            let handle = task.launch();
            self.running_tasks.insert(handle.id, handle);
        }
    }
}
