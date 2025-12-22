use heleny_proto::message::Message;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Monitor {
    rx: mpsc::Receiver<Message>,
}

impl Monitor {
    pub fn new(rx: mpsc::Receiver<Message>) -> Self {
        Self { rx }
    }

    pub async fn recv(&mut self) -> Option<Message> {
        self.rx.recv().await
    }
}
