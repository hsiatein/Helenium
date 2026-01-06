use std::collections::HashMap;
use std::collections::VecDeque;
use anyhow::Context;
use heleny_proto::ExecutorModel;
use heleny_proto::PlannerModel;
use heleny_proto::message::downcast;
use heleny_proto::name::CHAT_SERVICE;
use heleny_proto::name::TOOLKIT_SERVICE;
use heleny_service::ChatServiceMessage;
use heleny_service::ToolkitServiceMessage;
use heleny_service::get_from_config_service;
use tokio::sync::oneshot;
use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::TaskServiceMessage;
use heleny_proto::{message::AnyMessage, role::ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::resource::Resource;

mod task;
pub use task::*;
mod task_config;
pub use task_config::*;
use tracing::info;
use uuid::Uuid;

#[base_service(deps=["ConfigService","ChatService"])]
pub struct TaskService{
    endpoint:Endpoint,
    running_tasks:HashMap<Uuid,TaskHandle>,
    pending_tasks:VecDeque<Task>,
    config:TaskConfig,
}

#[derive(Debug)]
enum WorkerMessage{
    Finish{
        id:Uuid,
        success:bool
    },
    GetPlanner{
        feedback:oneshot::Sender<PlannerModel>,
    },
    GetManuals{
        tool_names:Vec<String>,
        feedback:oneshot::Sender<String>,
    },
    GetExecutor{
        feedback:oneshot::Sender<ExecutorModel>,
    }
}

#[async_trait]
impl Service for TaskService {
    type MessageType= TaskServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let config:TaskConfig=get_from_config_service(&endpoint).await?;
        let instance=Self {
            endpoint,
            running_tasks:HashMap::new(),
            pending_tasks:VecDeque::new(),
            config,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: TaskServiceMessage,
    ) -> Result<()>{
        match msg {
            TaskServiceMessage::AddTask { task_description }=>{
                let task=Task::new(Uuid::new_v4(), task_description, self.endpoint.create_sub_endpoint()?,self.config.max_working_loop);
                self.pending_tasks.push_back(task);
                self.launch_tasks();
            }
        }
        Ok(())
    }
    async fn stop(&mut self){

    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()>{
        let msg:WorkerMessage=downcast(msg)?;
        match msg {
            WorkerMessage::Finish { id, success }=>{
                let handle=self.running_tasks.remove(&id).context("没有此 ID 的任务")?;
                handle.handle.abort();
                self.launch_tasks();
                let log=handle.get_log()?;
                if success {
                    info!("任务 {} 成功: {:?}",id,log);
                }else {
                    info!("任务 {} 失败: {:?}",id,log);
                }
                self.endpoint.send(CHAT_SERVICE, ChatServiceMessage::TaskFinished { log }).await
            }
            WorkerMessage::GetPlanner { feedback }=>{
                self.endpoint.send(
                    CHAT_SERVICE,
                    ChatServiceMessage::GetPlanner { feedback },
                ).await
            }
            WorkerMessage::GetManuals { tool_names, feedback }=>{
                self.endpoint.send(TOOLKIT_SERVICE, ToolkitServiceMessage::GetManuals { tool_names, feedback }).await
            }
            WorkerMessage::GetExecutor { feedback }=>{
                self.endpoint.send(CHAT_SERVICE, ChatServiceMessage::GetExecutor { feedback }).await
            }
        }
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl TaskService {
    fn launch_tasks(&mut self){
        while self.running_tasks.len() < self.config.max_running_tasks {
            let Some(task) = self.pending_tasks.pop_front() else {
                return ;
            };
            let handle=task.launch();
            self.running_tasks.insert(handle.id, handle);
        }
    }
}
