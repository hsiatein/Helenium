use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use anyhow::Context;
use heleny_bus::endpoint::SubEndpoint;
use tokio::{task::JoinHandle};
use tracing::info;
use tracing::warn;
use uuid::Uuid;

use crate::WorkerMessage;

pub struct Task {
    id:Uuid,
    task_description:String,
    sender:SubEndpoint,
    log:Arc<Mutex<Vec<String>>>,
}

pub struct TaskHandle {
    pub id:Uuid,
    pub handle:JoinHandle<()>,
    pub log:Arc<Mutex<Vec<String>>>,
}

impl TaskHandle {
    pub fn get_log(&self)->Result<Vec<String>>{
        match self.log.lock() {
            Ok(log)=>{
                Ok(log.to_owned())
            }
            Err(e)=>{
                Err(anyhow::anyhow!("任务 {} 日志失效: {}",self.id,e))
            }
        }
    }
}

impl Task {
    pub fn new(id:Uuid, task_description:String, sender:SubEndpoint)->Self{
        Self { id, task_description, sender, log: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn launch(mut self)->TaskHandle{
        let id=self.id;
        let log=self.log.clone();
        info!("启动任务 {}, 描述: {}",id,self.task_description);
        let handle=tokio::spawn(async move {
            let success;
            match self.run().await {
                Ok(_)=>{
                    self.log(format!("任务成功"));
                    success=true;
                }
                Err(e)=>{
                    self.log(format!("任务失败: {}",e));
                    success=false;
                }
            };
            if let Err(e) = self.send(WorkerMessage::Finish { id:self.id, success }).await {
                warn!("发送任务结束信息失败: {}",e);
            };
        });
        TaskHandle { id, handle, log }
    }

    pub async fn run(&mut self)->Result<()>{
        Ok(())
    }

    async fn send(&self, msg:WorkerMessage)->Result<()>{
        self.sender.send(Box::new(msg)).await.context("发送消息给 Task Service 失败")
    }

    fn log(&self, text:String){
        match self.log.lock() {
            Ok(mut log)=>{
                log.push(text);
            }
            Err(e)=>{
                warn!("任务 {} 日志失效: {}",self.id,e);
            }
        };
    }
}