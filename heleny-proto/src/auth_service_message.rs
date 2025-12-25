use ed25519_dalek::Signature;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum AuthServiceMessage {
    GetChallenge {
        msg_sender: oneshot::Sender<[u8; 32]>,
    },
    Verify {
        msg: [u8; 32],
        signature: Signature,
        pass: oneshot::Sender<bool>,
    },
    Update,
}
