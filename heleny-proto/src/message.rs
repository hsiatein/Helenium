use anyhow::Result;
use std::any::Any;
use std::any::{self};
use std::fmt::Debug;
use uuid::Uuid;

use crate::role::ServiceRole;

/// Token消息 struct，定义Token消息的字段
#[derive(Debug)]
pub struct TokenMessage {
    pub target: String,
    pub token: Uuid,
    pub payload: Box<dyn AnyMessage>,
}

impl TokenMessage {
    pub fn new(target: String, token: Uuid, payload: Box<dyn AnyMessage>) -> Self {
        Self {
            target,
            token,
            payload,
        }
    }

    pub fn sign(self, name: String, role: ServiceRole) -> SignedMessage {
        SignedMessage::new(self.target, name, role, self.payload)
    }
}

/// 消息 struct，定义消息的字段
#[derive(Debug)]
pub struct SignedMessage {
    pub target: String,
    pub name: String,
    pub role: ServiceRole,
    pub payload: Box<dyn AnyMessage>,
}

impl SignedMessage {
    pub fn new(
        target: String,
        name: String,
        role: ServiceRole,
        payload: Box<dyn AnyMessage>,
    ) -> Self {
        Self {
            target,
            name,
            role,
            payload,
        }
    }
}

/// 服务消息类型 trait，用于定义服务对应的消息类型
pub trait AnyMessage: Send + Sync + Any + Debug {
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Any + Send + Sync + Debug> AnyMessage for T {
    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub fn downcast<T: Any>(msg: Box<dyn AnyMessage>) -> Result<T> {
    let msg = msg
        .as_any()
        .downcast::<T>()
        .map_err(|e| anyhow::anyhow!("Downcast 成 {} 消息失败: {:?}", any::type_name::<T>(), e))?;
    Ok(*msg)
}
