use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::HelenyFile;
use heleny_proto::HelenyFileType;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::FsServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use heleny_service::register_tool_factory;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::write;
use tokio::fs::{self};
use tokio::time::Instant;
use tracing::warn;

use crate::cache_entry::CacheEntry;
use crate::config::FsConfig;
use crate::tool::FsToolFactory;

mod cache_entry;
mod config;
mod tool;

#[base_service(deps=["ConfigService"])]
pub struct FsService {
    endpoint: Endpoint,
    _temp_dir: PathBuf,
    cache: HashMap<PathBuf, CacheEntry>,
}

#[async_trait]
impl Service for FsService {
    type MessageType = FsServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config: FsConfig = get_from_config_service(&endpoint).await?;
        tokio::fs::create_dir_all(&config.exchange_dir).await?;
        let exchange_dir = tokio::fs::canonicalize(&config.exchange_dir).await?;
        tokio::fs::create_dir_all(&config.temp_dir).await?;
        let _temp_dir = tokio::fs::canonicalize(&config.temp_dir).await?;
        let factory = FsToolFactory::new(endpoint.create_sender_endpoint(), exchange_dir);
        register_tool_factory(&endpoint, factory).await;
        let instance = Self {
            endpoint,
            _temp_dir,
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
                self.load(&abs_path).await?;
                let data = self.cache.get(&abs_path).context("未找到文件")?;
                if let HelenyFile::Text(data) = &data.content {
                    let _ = feedback.send(data.clone());
                };
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
                    content: HelenyFile::Text(content),
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
                    let data = CacheEntry::read_text(path).await?;
                    *entry = data;
                }
                Ok(())
            }
            FsServiceMessage::List { dir, feedback } => {
                let mut entries = fs::read_dir(dir).await?;
                let mut items = Vec::new();
                while let Some(entry) = entries.next_entry().await? {
                    let path = tokio::fs::canonicalize(entry.path()).await?;
                    items.push(path);
                }
                let _ = feedback.send(items);
                Ok(())
            }
            FsServiceMessage::Load { path, feedback } => {
                self.load(&path).await?;
                let _ = feedback.send(());
                Ok(())
            }
            FsServiceMessage::GetImage { path, feedback } => {
                let abs_path = tokio::fs::canonicalize(&path).await?;
                self.load(&abs_path).await?;
                let data = self.cache.get(&abs_path).context("未找到文件")?;
                if let HelenyFile::Image(data) = &data.content {
                    let _ = feedback.send(data.clone());
                };
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

impl FsService {
    async fn load(&mut self, path: &Path) -> Result<()> {
        let abs_path = tokio::fs::canonicalize(&path).await?;
        let modified = fs::metadata(&abs_path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        let data = match get_file_type(&abs_path) {
            HelenyFileType::Text => match self.cache.get(&abs_path) {
                Some(data) if data.last_modified == modified => return Ok(()),
                _ => CacheEntry::read_text(&abs_path).await?,
            },
            HelenyFileType::Image => match self.cache.get(&abs_path) {
                Some(data) if data.last_modified == modified => return Ok(()),
                _ => CacheEntry::read_image(&abs_path).await?,
            },
            HelenyFileType::Unknown => {
                return Err(anyhow::anyhow!("未知文件类型"));
            }
        };
        self.cache.insert(abs_path.clone(), data);
        Ok(())
    }
}

fn get_file_type(path: &Path) -> HelenyFileType {
    let Some(ext) = path.extension() else {
        return HelenyFileType::Unknown;
    };
    let Some(ext) = ext.to_str() else {
        return HelenyFileType::Unknown;
    };
    let image_exts = HashSet::from(["png", "jpg", "jpeg", "svg", "webp"]);
    if image_exts.contains(ext) {
        return HelenyFileType::Image;
    } else {
        return HelenyFileType::Text;
    }
}
