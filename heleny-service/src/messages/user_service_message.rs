use heleny_proto::{ConsentRequestion, ConsentRequestionFE, FrontendType};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum UserServiceMessage {
    Login(FrontendType),
    RequestConsent{
        body: ConsentRequestion,
    },
    ListConsentRequestions{
        feedback: oneshot::Sender<Vec<ConsentRequestionFE>>,
    },
}
