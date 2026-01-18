use heleny_proto::Embed;
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
    GetEmbedModel {
        base_url:String,
        model:String,
        api_key:String,
        feedback: oneshot::Sender<Box<dyn Embed>>
    },
    Reload,
}
