use std::fmt::Debug;
use anyhow::Result;
use async_trait::async_trait;

use crate::ToolIntent;

#[async_trait]
pub trait ToolFactory:Send {
    async fn create(&mut self)->Box<dyn Tool>;
}

#[async_trait]
pub trait Tool: Debug + Send + 'static {
    async fn invoke(&mut self,intent:ToolIntent)->Result<String>;
    async fn request_consent(&self,intent:ToolIntent)->bool;
}