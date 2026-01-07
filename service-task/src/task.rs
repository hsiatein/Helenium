use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use heleny_bus::endpoint::SubEndpoint;
use heleny_proto::ExecutorModel;
use heleny_proto::PlannerModel;
use heleny_service::Toolkit;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

use crate::WorkerMessage;

pub struct Task {
    id: Uuid,
    task_description: String,
    sender: SubEndpoint,
    max_working_loop: usize,
    current: usize,
    log: Arc<Mutex<Vec<String>>>,
}

pub struct TaskHandle {
    pub id: Uuid,
    pub handle: JoinHandle<()>,
    pub log: Arc<Mutex<Vec<String>>>,
}

impl TaskHandle {
    pub fn get_log(&self) -> Result<Vec<String>> {
        match self.log.lock() {
            Ok(log) => Ok(log.to_owned()),
            Err(e) => Err(anyhow::anyhow!("任务 {} 日志失效: {}", self.id, e)),
        }
    }
}

impl Task {
    pub fn new(
        id: Uuid,
        task_description: String,
        sender: SubEndpoint,
        max_working_loop: usize,
    ) -> Self {
        Self {
            id,
            task_description,
            sender,
            max_working_loop,
            current: 0,
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn launch(mut self) -> TaskHandle {
        let id = self.id;
        let log = self.log.clone();
        info!("启动任务 {}, 描述: {}", id, self.task_description);
        let handle = tokio::spawn(async move {
            let success;
            match self.run().await {
                Ok(_) => {
                    self.log(format!("任务成功"));
                    success = true;
                }
                Err(e) => {
                    self.log(format!("任务失败: {}", e));
                    success = false;
                }
            };
            if let Err(e) = self
                .send(WorkerMessage::Finish {
                    id: self.id,
                    success,
                })
                .await
            {
                warn!("发送任务结束信息失败: {}", e);
            };
        });
        TaskHandle { id, handle, log }
    }

    pub async fn run(&mut self) -> Result<()> {
        let (mut executor,mut toolkit) = self.preprocess().await?;
        let mut input = self.task_description.clone();
        while self.current < self.max_working_loop {
            let intent = match executor.get_intent(&input).await {
                Ok(intent) => intent,
                Err(e) => {
                    self.log(format!("获取 Intent 失败, 重试: {}", e));
                    self.current = self.current + 1;
                    continue;
                }
            };
            if intent.tool.is_none() && intent.command.is_none() {
                return Ok(());
            }
            let result=toolkit.invoke(intent).await;
            input=format!("<tool_result>{}</tool_result>",result);
            self.log(&input);
            self.current = self.current + 1;
        }
        let context = "达到最大工作循环限制";
        self.log(context);
        Err(anyhow::anyhow!(context))
    }

    async fn preprocess(&self) -> Result<(ExecutorModel,Toolkit)> {
        let planner = match self.get_planner().await {
            Ok(planner) => {
                self.log("成功获取 Planner");
                planner
            }
            Err(e) => {
                let context = format!("无法获取到所需工具说明书: {}", e);
                self.log(&context);
                return Err(anyhow::anyhow!(context));
            }
        };
        let tools_list = match planner.get_tools_list(&self.task_description).await {
            Ok(tools_list) => {
                self.log(format!("成功获取所需工具列表: {:?}", tools_list));
                tools_list
            }
            Err(e) => {
                let context = format!("获取所需工具列表失败: {}", e);
                self.log(&context);
                return Err(anyhow::anyhow!(context));
            }
        };
        let Some(tool_names) = tools_list.tools else {
            let context = "工具无法满足任务需求, 无法继续";
            self.log(context);
            return Err(anyhow::anyhow!(context));
        };
        let toolkit = match self.get_toolkit(tool_names).await {
            Ok(manuals) => {
                self.log("成功获取所需工具箱");
                manuals
            }
            Err(e) => {
                let context = format!("获取所需工具箱失败: {}", e);
                self.log(&context);
                return Err(anyhow::anyhow!(context));
            }
        };
        let executor = match self.get_executor().await {
            Ok(mut executor) => {
                executor.add_preset(toolkit.get_manuals());
                self.log("成功获取所需 Executor");
                executor
            }
            Err(e) => {
                let context = format!("获取所需 Executor 失败: {}", e);
                self.log(&context);
                return Err(anyhow::anyhow!(context));
            }
        };
        Ok((executor,toolkit))
    }

    async fn send(&self, msg: WorkerMessage) -> Result<()> {
        self.sender
            .send(Box::new(msg))
            .await
            .context("发送消息给 Task Service 失败")
    }

    fn log<T: Into<String>>(&self, text: T) {
        match self.log.lock() {
            Ok(mut log) => {
                log.push(text.into());
            }
            Err(e) => {
                warn!("任务 {} 日志失效: {}", self.id, e);
            }
        };
    }

    async fn get_planner(&self) -> Result<PlannerModel> {
        let (tx, rx) = oneshot::channel();
        self.send(WorkerMessage::GetPlanner { feedback: tx })
            .await?;
        rx.await.context("接收 Planner 失败")
    }

    async fn get_toolkit(&self, tool_names: Vec<String>) -> Result<Toolkit> {
        let (tx, rx) = oneshot::channel();
        self.send(WorkerMessage::GetToolkit { tool_names, task_id: self.id, task_description: self.task_description.clone(), feedback: tx })
        .await?;
        rx.await.context("接收 Manuals 失败")
    }

    async fn get_executor(&self) -> Result<ExecutorModel> {
        let (tx, rx) = oneshot::channel();
        self.send(WorkerMessage::GetExecutor { feedback: tx })
            .await?;
        rx.await.context("接收 Executor 失败")
    }

}
