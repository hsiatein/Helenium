use std::collections::HashMap;

use heleny_proto::ScheduledTask;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub enum ScheduleServiceMessage {
    AddTask {
        task: ScheduledTask,
    },
    ListTask {
        feedback: oneshot::Sender<HashMap<Uuid, ScheduledTask>>,
    },
    CancelTask {
        id: Uuid,
    },
}
