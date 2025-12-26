use heleny_proto::frontend_type::FrontendType;

#[derive(Debug)]
pub enum UserServiceMessage {
    Login(FrontendType),
}
