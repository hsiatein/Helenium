use heleny_proto::FrontendType;

#[derive(Debug)]
pub enum UserServiceMessage {
    Login(FrontendType),
}
