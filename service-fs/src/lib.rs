use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::CONFIG_STORAGE_DIR;
use heleny_proto::FS_SERVICE;
use heleny_proto::HelenyFile;
use heleny_proto::HelenyFileType;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_service::FsServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use heleny_service::import_from_config_service;
use heleny_service::register_tool_factory;
use tar::Archive;
use tar::Builder;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::write;
use tokio::fs::{self};
use tokio::time::Instant;
use tracing::warn;
use crc32fast;

use crate::cache_entry::CacheEntry;
use crate::cache_entry::make_thumbnail;
use crate::config::FsConfig;
use crate::tool::FsToolFactory;

mod cache_entry;
mod config;
mod tool;

#[base_service(deps=["ConfigService"])]
pub struct FsService {
    endpoint: Endpoint,
    temp_dir: PathBuf,
    cache: HashMap<PathBuf, CacheEntry>,
    archive: bool,
    archive_path: PathBuf,
    storage_dir: PathBuf,
    thumbnails: HashMap<PathBuf,(Uuid,SystemTime)>,
    thumbnails_dir: PathBuf,
    thumbnails_json: PathBuf,
    is_calculating: HashMap<Uuid,Vec<oneshot::Sender<Vec<u8>>>>,
}

#[async_trait]
impl Service for FsService {
    type MessageType = FsServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        // 读取配置
        let config: FsConfig = get_from_config_service(&endpoint).await?;
        let storage_dir:PathBuf=import_from_config_service(&endpoint, CONFIG_STORAGE_DIR).await?;
        let FsConfig { exchange_dir, temp_dir, archive, archive_path }=config;
        // 创建交换和临时目录
        tokio::fs::create_dir_all(&exchange_dir).await?;
        let exchange_dir = tokio::fs::canonicalize(exchange_dir).await?;
        tokio::fs::create_dir_all(&temp_dir).await?;
        let temp_dir = tokio::fs::canonicalize(temp_dir).await?;
        // 加载 tar 存储包
        if archive && let Ok(file) = fs::File::open(&archive_path).await {
            let mut a = Archive::new(file.into_std().await);
            if let Ok(_) = fs::create_dir_all(&storage_dir).await{
                if let Err(e)=a.unpack(&storage_dir){
                    warn!("解压失败: {}",e);
                };
            };
        }
        // 加载 thumbnails 
        let thumbnails_dir= storage_dir.join("thumbnails");
        let thumbnails_json=storage_dir.join("thumbnails.json");
        let thumbnails=match fs::read_to_string(&thumbnails_json).await {
            Ok(str)=> {
                serde_json::from_str(&str).unwrap_or(HashMap::new())
            }
            Err(_)=>HashMap::new(),
        };
        fs::create_dir_all(&thumbnails_dir).await?;
        // 注册 fs tool 的 factory
        let factory = FsToolFactory::new(endpoint.create_sender_endpoint(), exchange_dir);
        register_tool_factory(&endpoint, factory).await;
        // 实例化
        let instance = Self {
            endpoint,
            temp_dir,
            cache: HashMap::new(),
            archive,
            archive_path,
            thumbnails_dir,
            storage_dir,
            thumbnails,
            thumbnails_json,
            is_calculating: HashMap::new(),
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
                let abs_path = self.load(&path).await?;
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
            FsServiceMessage::GetOriginImage { path, feedback } => {
                let abs_path = self.load(&path).await?;
                let data = self.cache.get(&abs_path).context("未找到文件")?;
                if let HelenyFile::Image(data) = &data.content {
                    let _ = feedback.send(data.clone());
                };
                Ok(())
            }
            FsServiceMessage::GetImage { path, feedback }=>{
                self.load_thumbnail(&path,feedback).await
            }
            FsServiceMessage::TempFile { dir_name, data, file_name, feedback }=>{
                let hash=crc32fast::hash(&data);
                let dir=self.temp_dir.join(dir_name).join(hash.to_string());
                fs::create_dir_all(&dir).await?;
                let path=dir.join(file_name);
                fs::write(&path, data).await?;
                let _=feedback.send(path);
                Ok(())
            }
            FsServiceMessage::WriteBytes { path, data }=>{
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                write(&path, data).await?;
                Ok(())
            }
            FsServiceMessage::ReadBytes { path, feedback }=>{
                let data=fs::read(path).await?;
                let _ = feedback.send(data);
                Ok(())
            }
            FsServiceMessage::NewThumbnail { id, origin_path, last_modified, thumbnail }=>{
                let clients=self.is_calculating.remove(&id).context("没人等待此 thumbnail")?;
                for client in clients {
                    let _=client.send(thumbnail.clone());
                }
                self.thumbnails.insert(origin_path,(id,last_modified));
                let thumbnail_path=self.thumbnails_dir.join(id.to_string()+".jpg");
                fs::write(&thumbnail_path, thumbnail).await?;
                self.load(&thumbnail_path).await?;
                Ok(())
            }
        }
    }
    async fn stop(&mut self) {
        if let Ok(json)=serde_json::to_string(&self.thumbnails) {
            if let Err(e) = fs::write(&self.thumbnails_json, json ).await {
                warn!("保存 thumbnails 映射失败: {}",e);
            };
        }
        if self.archive && let Ok(file) = fs::File::create(&self.archive_path).await {
            let mut a = Builder::new(file.into_std().await);
            match a.append_dir_all(".",&self.storage_dir) {
                Ok(_)=>{
                    info!("创建 storage 归档成功");
                    if let Err(e)=fs::remove_dir_all(&self.storage_dir).await {
                        warn!("删除 storage 目录失败: {}",e);
                    };
                }
                Err(e)=>{
                    warn!("归档失败: {}",e);
                }
            }
        };
    }
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
    async fn load(&mut self, path: &Path) -> Result<PathBuf> {
        let abs_path = tokio::fs::canonicalize(&path).await?;
        let modified = fs::metadata(&abs_path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        let data = match get_file_type(&abs_path) {
            HelenyFileType::Text => match self.cache.get(&abs_path) {
                Some(data) if data.last_modified == modified => return Ok(abs_path),
                _ => CacheEntry::read_text(&abs_path).await?,
            },
            HelenyFileType::Image => match self.cache.get(&abs_path) {
                Some(data) if data.last_modified == modified => return Ok(abs_path),
                _ => CacheEntry::read_image(&abs_path).await?,
            },
            HelenyFileType::Unknown => {
                return Err(anyhow::anyhow!("未知文件类型"));
            }
        };
        self.cache.insert(abs_path.clone(), data);
        Ok(abs_path)
    }

    async fn load_thumbnail(&mut self, path:&PathBuf,feedback: oneshot::Sender<Vec<u8>>)->Result<()>{
        let abs_path = tokio::fs::canonicalize(path).await?;
        let last_modified = fs::metadata(&abs_path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        if let Some((id,time))=self.thumbnails.get(&abs_path) && *time==last_modified{
            let thumbnail_path=self.thumbnails_dir.join(id.to_string()+".jpg");
            if let Some(entry)=self.cache.get(&thumbnail_path) && let HelenyFile::Image(image)=entry.content.clone() {
                info!("命中 thumbnail 缓存");
                let _=feedback.send(image);
                return Ok(());
            }
            if let Ok(entry)=CacheEntry::read_image(&thumbnail_path).await && let HelenyFile::Image(image)=entry.content.clone() {
                info!("读取 thumbnail 加入缓存");
                self.cache.insert(thumbnail_path.clone(), entry);
                let _=feedback.send(image);
                return Ok(());
            }
        }
        info!("开始计算 thumbnail");
        let id=Uuid::new_v4();
        self.is_calculating.entry(id).or_insert(Vec::new()).push(feedback);
        let sender=self.endpoint.create_sender_endpoint();
        let _: JoinHandle<Result<(), anyhow::Error>>=tokio::spawn(async move {
            let (tx,rx)=oneshot::channel();
            sender.send(FS_SERVICE, FsServiceMessage::GetOriginImage { path: abs_path.clone(), feedback:tx }).await?;
            let image=rx.await?;
            let thumbnail=tokio::task::spawn_blocking({
                move || make_thumbnail(&image, 256)
            })
            .await?.context("计算 thumbnail 失败")?;
            sender.send(FS_SERVICE, FsServiceMessage::NewThumbnail { id, origin_path: abs_path, last_modified, thumbnail }).await?;
            Ok(())
        });
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
