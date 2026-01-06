use std::collections::HashMap;

use heleny_bus::endpoint::Endpoint;
use heleny_proto::Tool;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub enum ToolkitServiceMessage {
    GetIntro {
        feedback: oneshot::Sender<String>,
    },
    GetManuals {
        tool_names: Vec<String>,
        feedback: oneshot::Sender<String>,
    },
    GetToolkit {
        tool_names: Vec<String>,
        task_id: Uuid,
        task_description: String,
        feedback: oneshot::Sender<Toolkit>,
    }
}

#[derive(Debug)]
pub struct Toolkit {
    task_id:Uuid,
    task_description: String,
    endpoint:Endpoint,
    tools: HashMap<String,Box<dyn Tool>>,
}