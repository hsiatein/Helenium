use heleny_proto::message::Message;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Midware {
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
}

impl Midware {
    pub fn new(tx: mpsc::Sender<Message>, rx: mpsc::Receiver<Message>) -> Self {
        Self { tx, rx }
    }
    pub async fn recv(&mut self) -> Option<Message> {
        self.rx.recv().await
    }
    pub async fn send(&mut self, msg: Message) {
        let _ = self.tx.send(msg).await;
    }
}
