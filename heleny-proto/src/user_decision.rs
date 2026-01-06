use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub struct ConsentRequestion {
    pub task_id:Uuid,
    pub task_description:String,
    pub reason:String,
    pub tool:String,
    pub command:String,
    pub args:HashMap<String,String>,
    pub feedback: oneshot::Sender<()>,
}

impl ConsentRequestion {
    pub fn to_frontend(self,request_id:Uuid)->(ConsentRequestionFE,oneshot::Sender<()>) {
        let ConsentRequestion { task_id, task_description, reason, tool, command, args, feedback }=self;
        let requestion_fe=ConsentRequestionFE { request_id, task_id, task_description, reason, tool, command, args };
        (requestion_fe,feedback)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequestionFE {
    pub request_id:Uuid,
    pub task_id:Uuid,
    pub task_description:String,
    pub reason:String,
    pub tool:String,
    pub command:String,
    pub args:HashMap<String,String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserDecision {
    ConsentRequestion(ConsentRequestionFE),
}