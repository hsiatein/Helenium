use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::{CanRequestConsent, FS_SERVICE, HelenyTool, HelenyToolFactory, ToolArg};
use heleny_service::FsServiceMessage;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct FsToolFactory {
    endpoint:Endpoint,
    exchange_dir:PathBuf,
}

impl FsToolFactory {
    pub fn new(endpoint:Endpoint, exchange_dir:PathBuf,)->Self{
        Self {endpoint,exchange_dir}
    }
}

#[async_trait]
impl HelenyToolFactory for FsToolFactory {
    fn name(&self)->String{
        "file".into()
    }
    async fn create(&mut self)->Result<Box<dyn HelenyTool>> {
        let tool=FsTool::new(self.endpoint.create_sender_endpoint(), self.exchange_dir.clone());
        Ok(Box::new(tool))
    }
}

#[derive(Debug)]
pub struct FsTool {
    endpoint:Endpoint,
    exchange_dir:PathBuf,
}

impl FsTool {
    pub fn new(endpoint:Endpoint, exchange_dir:PathBuf,)->Self{
        Self {endpoint,exchange_dir}
    }
}

#[async_trait]
impl HelenyTool for FsTool {
    async fn invoke(&mut self,command:String,args:Vec<ToolArg>,request:Box<&dyn CanRequestConsent>)->Result<String>{
        match command.as_str() {
            "ls-exchange"=>{
                let (tx,rx)=oneshot::channel();
                self.endpoint.send(FS_SERVICE, FsServiceMessage::List { dir: self.exchange_dir.clone(), feedback: tx }).await?;
                let files=rx.await?;
                serde_json::to_string(&files).context("解析目录内容列表失败")
            }
            cmd=>{
                Err(anyhow::anyhow!("未知命令: {}",cmd))
            }
        }
    }
}