use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::DISPLAY_MESSAGES;
use heleny_proto::DisplayMessage;
use heleny_proto::MemoryEntry;
use heleny_proto::Resource;
use heleny_proto::ResourcePayload;
use heleny_proto::ServiceRole;
use heleny_service::MemoryServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use heleny_service::publish_resource;
use tokio::fs;
use tokio::sync::watch;
use tokio::time::Instant;
use tracing::debug;
use tracing::info;

use crate::config::MemoryConfig;
use crate::memory_db::MemoryDb;

mod config;
mod memory_db;

#[base_service(deps=["ConfigService","HubService"])]
pub struct MemoryService {
    endpoint: Endpoint,
    config: MemoryConfig,
    short_term: VecDeque<MemoryEntry>,
    memory_db: MemoryDb,
    publisher: watch::Sender<ResourcePayload>,
}

#[derive(Debug)]
enum _WorkerMessage {}

#[async_trait]
impl Service for MemoryService {
    type MessageType = MemoryServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config: MemoryConfig = get_from_config_service(&endpoint).await?;
        let dir = PathBuf::from(&config.storage_dir);
        fs::create_dir_all(&dir).await.context("创建储存目录失败")?;
        let storage_path = dir.join("memory.db");
        // 新建 MemoryDb
        let memory_db = MemoryDb::new(&storage_path).await?;
        info!("已连接 Memory DB");
        // 发布最新消息
        let (tx, rx) = watch::channel(ResourcePayload::DisplayMessages {
            new: true,
            messages: Vec::new(),
        });
        publish_resource(&endpoint, DISPLAY_MESSAGES, rx).await?;
        // 新建实例
        let mut short_term = VecDeque::with_capacity(config.short_term_length);
        let _short_term = memory_db
            .get_chat_messages(config.short_term_length as i64)
            .await?;
        short_term.extend(_short_term);
        debug!(
            "短期记忆: {:?}，长度: {}",
            short_term, config.short_term_length as i64
        );
        let instance = Self {
            endpoint,
            config,
            short_term,
            memory_db,
            publisher: tx,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: MemoryServiceMessage,
    ) -> Result<()> {
        match msg {
            MemoryServiceMessage::Post { entry } => {
                if self.short_term.len() >= self.config.short_term_length {
                    self.short_term.pop_front();
                }
                self.short_term.push_back(entry.clone());
                let id = self.memory_db.save_entry(entry.clone()).await?;
                let display_message = DisplayMessage::new(id, entry);
                self.publisher
                    .send(ResourcePayload::DisplayMessages {
                        new: true,
                        messages: vec![display_message],
                    })
                    .context("更新 DisplayMessages 失败")
            }
            MemoryServiceMessage::GetChatMemories { feedback } => {
                let mut chat_memories = Vec::new();
                for entry in &self.short_term {
                    chat_memories.push(entry.to_chat_message()?);
                }
                let _ = feedback.send(chat_memories);
                Ok(())
            }
            MemoryServiceMessage::Get {
                id_upper_bound,
                feedback,
            } => {
                let result = self
                    .memory_db
                    .get_display_messages(id_upper_bound, self.config.display_length as i64)
                    .await?;
                let _ = feedback.send(result);
                Ok(())
            }
        }
    }
    async fn stop(&mut self) {
        self.memory_db.close().await;
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

impl MemoryService {}
