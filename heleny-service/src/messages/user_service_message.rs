use heleny_proto::ConsentRequestion;
use heleny_proto::ConsentRequestionFE;
use heleny_proto::FrontendType;
use tokio::sync::oneshot;
use uuid::Uuid;

#[derive(Debug)]
pub enum UserServiceMessage {
    Login(FrontendType),
    Logout,
    RequestConsent {
        body: ConsentRequestion,
    },
    ListConsentRequestions {
        feedback: oneshot::Sender<Vec<ConsentRequestionFE>>,
    },
    MakeDecision {
        req_id: Uuid,
        approval: bool,
    },
}
