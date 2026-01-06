use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ToolkitServiceMessage {
    GetIntro {
        feedback:oneshot::Sender<String>,
    },
    GetManuals {
        tool_names: Vec<String>,
        feedback:oneshot::Sender<String>,
    }
}