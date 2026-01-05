use heleny_proto::PlannerModel;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ChatServiceMessage {
    Chat{
        message:String,
    },
    GetPlanner{
        feedback:oneshot::Sender<PlannerModel>
    }
}
