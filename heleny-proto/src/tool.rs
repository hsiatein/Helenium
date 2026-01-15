use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;

#[async_trait]
pub trait HelenyToolFactory: Debug + Send + Sync + 'static {
    fn name(&self) -> String;
    async fn create(&mut self) -> Result<Box<dyn HelenyTool>>;
}

#[async_trait]
pub trait HelenyTool: Debug + Send + 'static {
    async fn invoke(
        &mut self,
        command: String,
        args: HashMap<String, Value>,
        request: Box<&dyn CanRequestConsent>,
    ) -> Result<String>;
}

#[async_trait]
pub trait CanRequestConsent: Sync {
    async fn request_consent(&self, description: String) -> Result<()>;
}


pub struct TestCanRequestConsent {
}

#[async_trait]
impl CanRequestConsent for TestCanRequestConsent {
    async fn request_consent(&self, _: String) -> Result<()>{
        Ok(())
    }
}

impl TestCanRequestConsent {
    pub fn new()->Self{
        Self{}
    }
}