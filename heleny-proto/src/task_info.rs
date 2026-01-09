use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug,Serialize,Clone, Deserialize)]
pub struct TaskLog {
    pub task_description: String,
    pub log: Vec<String>,
    pub status: TaskStatus,
}

#[derive(Debug,Serialize,Clone, Deserialize)]
pub struct TaskAbstract {
    pub id:Uuid,
    pub task_description: String,
    pub status: TaskStatus,
}

#[derive(Debug,Serialize,Clone, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Canceled,
    Success,
    Fail,
}

impl TaskLog {
    pub fn new(task_description: String)->Self{
        Self { task_description, log: Vec::new(), status: TaskStatus::Pending }
    }
    pub fn log(&mut self,context: String){
        self.log.push(context);
    }
    pub fn get_log(&self)->Vec<String>{
        self.log.clone()
    }
}