use heleny_proto::TokenMessage;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Midware {
    tx: mpsc::Sender<TokenMessage>,
    rx: mpsc::Receiver<TokenMessage>,
}

impl Midware {
    pub fn new(tx: mpsc::Sender<TokenMessage>, rx: mpsc::Receiver<TokenMessage>) -> Self {
        Self { tx, rx }
    }
    pub async fn recv(&mut self) -> Option<TokenMessage> {
        self.rx.recv().await
    }
    pub async fn send(&mut self, msg: TokenMessage) {
        let _ = self.tx.send(msg).await;
    }
}
