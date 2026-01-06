use heleny_proto::{ConsentRequestion, FrontendType};

#[derive(Debug)]
pub enum UserServiceMessage {
    Login(FrontendType),
    RequestConsent{
        body: ConsentRequestion,
    },
}
