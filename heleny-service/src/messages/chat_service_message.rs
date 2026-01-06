use heleny_proto::ExecutorModel;
use heleny_proto::PlannerModel;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ChatServiceMessage {
    Chat {
        message: String,
    },
    TaskFinished {
        log: Vec<String>,
    },
    GetPlanner {
        feedback: oneshot::Sender<PlannerModel>,
    },
    GetExecutor {
        feedback: oneshot::Sender<ExecutorModel>,
    },
}
