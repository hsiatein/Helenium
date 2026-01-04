use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::message::AnyMessage;
use heleny_proto::resource::Resource;
use heleny_proto::role::ServiceRole;
use heleny_service::FsServiceMessage;
use heleny_service::Service;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::write;
use tokio::fs::{self};
use tokio::time::Instant;
use tracing::warn;

use crate::cache_entry::CacheEntry;

mod cache_entry;

#[base_service(deps=[])]
pub struct FsService {
    endpoint: Endpoint,
    cache: HashMap<PathBuf, CacheEntry>,
}

#[async_trait]
impl Service for FsService {
    type MessageType = FsServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let instance = Self {
            endpoint,
            cache: HashMap::new(),
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: FsServiceMessage,
    ) -> Result<()> {
        match msg {
            FsServiceMessage::Read { path, feedback } => {
                let abs_path = tokio::fs::canonicalize(&path).await?;
                let modified = fs::metadata(&abs_path)
                    .await
                    .context("获取文件元数据失败")?
                    .modified()
                    .context("获取文件修改时间失败")?;
                let data = match self.cache.get(&abs_path) {
                    Some(data) if data.last_modified == modified => data.content.clone(),
                    _ => {
                        let data = CacheEntry::read(&abs_path).await?;
                        self.cache.insert(abs_path.clone(), data.clone());
                        data.content
                    }
                };
                let _ = feedback.send(data);
                Ok(())
            }
            FsServiceMessage::Write {
                path,
                content,
                feedback,
            } => {
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                write(&path, content.clone()).await?;

                let abs_path = tokio::fs::canonicalize(path).await?;
                let modified = fs::metadata(&abs_path).await?.modified()?;

                let entry = CacheEntry {
                    content,
                    last_modified: modified,
                };
                self.cache.insert(abs_path, entry);

                let _ = feedback.send(());
                Ok(())
            }
            FsServiceMessage::Update => {
                for (path, entry) in &mut self.cache {
                    let metadata = match fs::metadata(path).await {
                        Ok(metadata) => metadata,
                        Err(e) => {
                            warn!("获取元数据失败: {}", e);
                            continue;
                        }
                    };
                    let file_modified = match metadata.modified() {
                        Ok(modified) => modified,
                        Err(e) => {
                            warn!("获取修改时间失败: {}", e);
                            continue;
                        }
                    };
                    if file_modified == entry.last_modified {
                        continue;
                    }
                    let data = CacheEntry::read(path).await?;
                    *entry = data;
                }
                Ok(())
            }
            FsServiceMessage::List { dir, feedback }=>{
                let mut entries=fs::read_dir(dir).await?;
                let mut items=vec![];
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    items.push(path);
                }
                let _=feedback.send(items);
                Ok(())
            }
        }
    }
    async fn stop(&mut self) {}
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()> {
        Ok(())
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl FsService {}
