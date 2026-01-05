use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ToolkitServiceMessage {
    GetIntro {
        feedback:oneshot::Sender<String>,
    },
    GetManual {
        names: Vec<String>,
        feedback:oneshot::Sender<String>,
    }
}