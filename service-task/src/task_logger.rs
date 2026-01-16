use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use heleny_proto::ResourcePayload;
use heleny_proto::TaskAbstract;
use heleny_proto::TaskLog;
use heleny_proto::TaskStatus;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tracing::warn;
use uuid::Uuid;

pub struct TaskLogger {
    task_logs: HashMap<Uuid, TaskLog>,
    subscriber: HashMap<Uuid, Vec<mpsc::Sender<TaskLog>>>,
    watch_sender: watch::Sender<ResourcePayload>,
    running: bool,
}

impl TaskLogger {
    pub fn new(watch_sender: watch::Sender<ResourcePayload>) -> Self {
        Self {
            task_logs: HashMap::new(),
            subscriber: HashMap::new(),
            watch_sender,
            running: true,
        }
    }

    pub async fn handle_message(&mut self, msg: TaskLoggerMessage) -> Result<()> {
        match msg {
            TaskLoggerMessage::Log { id, context } => {
                let log = self.task_logs.get_mut(&id).context("没有此日志")?;
                log.log(context);
                if let Some(subs) = self.subscriber.get_mut(&id) {
                    subs.retain(|sub| !sub.is_closed());
                    for sub in subs {
                        if let Err(e) = sub.send(log.clone()).await {
                            warn!("发送任务日志给订阅者失败: {}", e);
                        }
                    }
                }
                Ok(())
            }
        }
    }

    pub async fn handle_command(&mut self, cmd: TaskLoggerCommand) -> Result<()> {
        match cmd {
            TaskLoggerCommand::AddTask { id, description } => {
                self.task_logs.insert(id, TaskLog::new(description));
                self.watch_sender.send(ResourcePayload::TaskAbstract {
                    task_abstracts: self.get_abstracts(),
                })?;
                Ok(())
            }
            TaskLoggerCommand::SetStatus { id, status } => {
                let log = self.task_logs.get_mut(&id).context("没有此日志")?;
                log.status = status;
                self.watch_sender.send(ResourcePayload::TaskAbstract {
                    task_abstracts: self.get_abstracts(),
                })?;
                Ok(())
            }
            TaskLoggerCommand::Subscribe { id, sender } => {
                let subs = self.subscriber.entry(id).or_insert(Vec::new());
                if let Some(log) = self.task_logs.get(&id) {
                    let _ = sender.send(log.clone()).await;
                }
                subs.push(sender);
                Ok(())
            }
            TaskLoggerCommand::Stop => {
                self.running = false;
                Ok(())
            }
            TaskLoggerCommand::GetLog { id, feedback } => {
                let log = self.task_logs.get(&id).context("没有此日志")?.clone();
                let _ = feedback.send(log);
                Ok(())
            }
        }
    }

    pub fn get_abstracts(&self) -> Vec<TaskAbstract> {
        self.task_logs
            .iter()
            .map(|(id, log)| TaskAbstract {
                id: *id,
                task_description: log.task_description.clone(),
                status: log.status.clone(),
            })
            .collect()
    }
}

pub struct TaskLoggerHandle {
    log_tx: mpsc::Sender<TaskLoggerMessage>,
    handle_tx: mpsc::Sender<TaskLoggerCommand>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl TaskLoggerHandle {
    pub fn new(
        log_tx: mpsc::Sender<TaskLoggerMessage>,
        handle_tx: mpsc::Sender<TaskLoggerCommand>,
        handle: tokio::task::JoinHandle<()>,
    ) -> Self {
        Self {
            log_tx,
            handle_tx,
            handle: Some(handle),
        }
    }

    pub fn get_log_sender(&self) -> mpsc::Sender<TaskLoggerMessage> {
        self.log_tx.clone()
    }

    pub async fn get_log(&self, id: Uuid) -> Result<TaskLog> {
        let (tx, rx) = oneshot::channel::<TaskLog>();
        self.handle_tx
            .send(TaskLoggerCommand::GetLog { id, feedback: tx })
            .await?;
        let log = rx.await?;
        Ok(log)
    }

    pub async fn add_task(&self, id: Uuid, description: String) -> Result<()> {
        self.handle_tx
            .send(TaskLoggerCommand::AddTask { id, description })
            .await?;
        Ok(())
    }

    pub async fn set_status(&self, id: Uuid, status: TaskStatus) -> Result<()> {
        self.handle_tx
            .send(TaskLoggerCommand::SetStatus { id, status })
            .await?;
        Ok(())
    }

    pub async fn subscribe(&self, id: Uuid, sender: mpsc::Sender<TaskLog>) -> Result<()> {
        self.handle_tx
            .send(TaskLoggerCommand::Subscribe { id, sender })
            .await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.handle_tx.send(TaskLoggerCommand::Stop).await?;
        let handle = self.handle.take().context("获取 handle 失败")?;
        handle.await?;
        Ok(())
    }
}

pub async fn launch_task_logger() -> (TaskLoggerHandle, watch::Receiver<ResourcePayload>) {
    let (log_tx, mut log_rx) = mpsc::channel::<TaskLoggerMessage>(32);
    let (handle_tx, mut handle_rx) = mpsc::channel::<TaskLoggerCommand>(32);
    let (watch_tx, watch_rx) = watch::channel::<ResourcePayload>(ResourcePayload::TaskAbstract {
        task_abstracts: Vec::new(),
    });
    let mut task_logger = TaskLogger::new(watch_tx);
    let handle = tokio::spawn(async move {
        while task_logger.running {
            tokio::select! {
                Some(msg)=log_rx.recv()=>{
                    if let Err(e)=task_logger.handle_message(msg).await{
                        warn!("处理任务日志消息失败: {}",e);
                    }
                }
                Some(cmd)=handle_rx.recv()=>{
                    if let Err(e)=task_logger.handle_command(cmd).await{
                        warn!("处理任务日志命令失败: {}",e);
                    }
                }
            }
        }
    });
    (TaskLoggerHandle::new(log_tx, handle_tx, handle), watch_rx)
}

pub enum TaskLoggerMessage {
    Log { id: Uuid, context: String },
}

pub enum TaskLoggerCommand {
    AddTask {
        id: Uuid,
        description: String,
    },
    SetStatus {
        id: Uuid,
        status: TaskStatus,
    },
    Subscribe {
        id: Uuid,
        sender: mpsc::Sender<TaskLog>,
    },
    Stop,
    GetLog {
        id: Uuid,
        feedback: oneshot::Sender<TaskLog>,
    },
}
