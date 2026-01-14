use heleny_proto::TaskLog;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub enum TaskServiceMessage {
    AddTask {
        task_description: String,
    },
    CancelTask {
        id: Uuid,
    },
    SubscribeTaskLogs {
        id: Uuid,
        sender: mpsc::Sender<TaskLog>,
    },
}
