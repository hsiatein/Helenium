use std::any::Any;
use uuid::Uuid;

/// 消息 struct，定义消息的字段
pub struct Message {
    pub target: &'static str,
    pub token: Option<Uuid>,
    pub payload: Box<dyn AnyMessage>,
}

impl Message {
    pub fn new(target: &'static str, token: Option<Uuid>, payload: Box<dyn AnyMessage>) -> Self {
        Self { target, token, payload }
    }
}

/// 服务消息类型 trait，用于定义服务对应的消息类型
pub trait AnyMessage: Send + Sync + Any {
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Any + Send + Sync> AnyMessage for T {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}