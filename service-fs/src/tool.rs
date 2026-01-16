use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::CanRequestConsent;
use heleny_proto::ChatRole;
use heleny_proto::FS_SERVICE;
use heleny_proto::HelenyTool;
use heleny_proto::HelenyToolFactory;
use heleny_proto::MEMORY_SERVICE;
use heleny_proto::get_tool_arg;
use heleny_service::FsServiceMessage;
use heleny_service::MemoryServiceMessage;
use pathdiff::diff_paths;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::canonicalize;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct FsToolFactory {
    endpoint: Endpoint,
    exchange_dir: PathBuf,
}

impl FsToolFactory {
    pub fn new(endpoint: Endpoint, exchange_dir: PathBuf) -> Self {
        Self {
            endpoint,
            exchange_dir,
        }
    }
}

#[async_trait]
impl HelenyToolFactory for FsToolFactory {
    fn name(&self) -> String {
        "file".into()
    }
    async fn create(&mut self) -> Result<Box<dyn HelenyTool>> {
        let tool = FsTool::new(
            self.endpoint.create_sender_endpoint(),
            self.exchange_dir.clone(),
        );
        Ok(Box::new(tool))
    }
}

#[derive(Debug)]
pub struct FsTool {
    endpoint: Endpoint,
    exchange_dir: PathBuf,
}

impl FsTool {
    pub fn new(endpoint: Endpoint, exchange_dir: PathBuf) -> Self {
        Self {
            endpoint,
            exchange_dir,
        }
    }
}

#[async_trait]
impl HelenyTool for FsTool {
    async fn invoke(
        &mut self,
        command: String,
        mut args: HashMap<String, Value>,
        _request: Box<&dyn CanRequestConsent>,
    ) -> Result<String> {
        match command.as_str() {
            "ls-exchange" => {
                let (tx, rx) = oneshot::channel();
                self.endpoint
                    .send(
                        FS_SERVICE,
                        FsServiceMessage::List {
                            dir: self.exchange_dir.clone(),
                            feedback: tx,
                        },
                    )
                    .await?;
                let files: Vec<PathBuf> = rx
                    .await?
                    .into_iter()
                    .filter_map(|file| diff_paths(file, &self.exchange_dir))
                    .collect();
                serde_json::to_string(&files).context("解析目录内容列表失败")
            }
            "load" => {
                let path: PathBuf = get_tool_arg(&mut args, "path")?;
                let path = match canonicalize(self.exchange_dir.join(path)).await {
                    Ok(path) => path,
                    Err(e) => {
                        return Err(anyhow::anyhow!("路径正则化失败: {}", e));
                    }
                };
                let (tx, rx) = oneshot::channel();
                self.endpoint
                    .send(FS_SERVICE, FsServiceMessage::Load { path, feedback: tx })
                    .await?;
                rx.await?;
                Ok("加载完成".into())
            }
            "send" => {
                let path: PathBuf = get_tool_arg(&mut args, "path")?;
                let path = match canonicalize(self.exchange_dir.join(path)).await {
                    Ok(path) => path,
                    Err(e) => {
                        return Err(anyhow::anyhow!("路径正则化失败: {}", e));
                    }
                };
                self.endpoint
                    .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role: ChatRole::Assistant, content: path.into() })
                    .await?;
                Ok("发送完成".into())
            }
            cmd => Err(anyhow::anyhow!("未知命令: {}", cmd)),
        }
    }
}
