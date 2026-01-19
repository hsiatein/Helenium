use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::anyhow;
use heleny_proto::CHAT_SERVICE;
use heleny_proto::CONFIG_STORAGE_DIR;
use heleny_proto::Embed;
use heleny_proto::Embedding;
use heleny_proto::FS_SERVICE;
use heleny_proto::MEMORY_SERVICE;
use heleny_service::ChatServiceMessage;
use heleny_service::FsServiceMessage;
use heleny_service::MemoryServiceMessage;
use heleny_service::get_from_config_service;
use heleny_service::import_from_config_service;
use serde::Deserialize;
use tokio::sync::oneshot;
use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::EmbedServiceMessage;
use heleny_proto::{AnyMessage, ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::Resource;
use hnsw_rs::prelude::*;
use tracing::info;
use tracing::warn;

#[base_service(deps=["ChatService","ConfigService","MemoryService","FsService"])]
pub struct EmbedService{
    endpoint:Endpoint,
    embed_model:Box<dyn Embed>,
    dimensions: u32,
    hnsw:Hnsw<'static,f32, DistCosine>,
    filter:EmbeddingFilter,
    filter_path:PathBuf,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for EmbedService {
    type MessageType= EmbedServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let config:Config=get_from_config_service(&endpoint).await?;
        let storage_dir:PathBuf=import_from_config_service(&endpoint, CONFIG_STORAGE_DIR).await?;
        let Config { base_url, model, api_key_env_var, dimensions }=config;
        // 获取 embed 模型
        let api_key=std::env::var(api_key_env_var).unwrap_or("".into());
        let (tx,rx)=oneshot::channel();
        endpoint.send(CHAT_SERVICE, ChatServiceMessage::GetEmbedModel { base_url, model, api_key, feedback: tx }).await?;
        let embed_model=rx.await?;
        // filter
        let filter_path=storage_dir.join("filter.rkyv");
        let (tx,rx)=oneshot::channel();
        let _=endpoint.send(FS_SERVICE, FsServiceMessage::ReadBytes { path: filter_path.clone(), feedback: tx }).await;
        let filter=match rx.await {
            Ok(bytes)=>{
                let archived: &<EmbeddingFilter as rkyv::Archive>::Archived = rkyv::access::<<EmbeddingFilter as rkyv::Archive>::Archived,rkyv::rancor::Error>(&bytes)?;
                let roundtrip: EmbeddingFilter = rkyv::deserialize::<EmbeddingFilter,rkyv::rancor::Error>(archived)?;
                roundtrip
            }
            Err(_)=>{
                EmbeddingFilter::new()
            }
        };
        info!("读取到 {} 条 embedding 信息",filter.embeddings.len());
        // hnsw
        let hnsw=Hnsw::new(24, 100000, 16, 400, DistCosine {});
        hnsw.parallel_insert_slice(&filter.embeddings.iter().map(|(id,vec)| (vec.as_ref(),(*id) as usize)).collect::<Vec<_>>());
        // 实例化
        endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::SetEmbedAvailable { available: true }).await?;
        let instance=Self {
            endpoint,
            embed_model,
            dimensions,
            hnsw,
            filter,
            filter_path,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: EmbedServiceMessage,
    ) -> Result<()>{
        match msg {
            EmbedServiceMessage::Embed { id, content }=>{
                if self.filter.embeddings.contains_key(&id) {
                    return Ok(());
                }
                let embedding=self.embed(vec![content]).await?.pop().context("未获取到向量")?;
                self.hnsw.insert((embedding.as_ref(),id as usize));
                self.filter.embeddings.insert(id,embedding);
            }
            EmbedServiceMessage::EmbedBatch { batch }=>{
                if batch.is_empty() {
                    return Ok(());
                }
                let mut messages=Vec::new();
                let batch_ids:Vec<_>=batch.into_iter().filter_map(|(id,msg)| {
                    if self.filter.embeddings.contains_key(&id) {
                        return None;
                    }
                    messages.push(msg);
                    Some(id)
                }).collect();
                let embeddings=self.embed(messages).await?;
                if embeddings.len()!=batch_ids.len() {
                    return Err(anyhow!("embeddings 和 batch_ids 长度对不上"));
                }
                info!("添加 {:?} 的embedding", batch_ids);
                embeddings.into_iter().zip(batch_ids).for_each(|(embedding,id)|{
                    self.hnsw.insert((embedding.as_ref(),id as usize));
                    self.filter.embeddings.insert(id,embedding);
                });
            }
            EmbedServiceMessage::Delete { id }=>{
                self.filter.embeddings.remove(&id);
            }
            EmbedServiceMessage::Search { content, num, feedback }=>{
                let embedding=self.embed(vec![content]).await?.pop().context("未获取到向量")?;
                let neighbours=self.hnsw.search_filter(embedding.as_ref(), num, 16, Some(&self.filter));
                let nbrs=neighbours.into_iter().map(|nbr| {
                    nbr.d_id as i64
                }).collect();
                let _=feedback.send(nbrs);
            }
            EmbedServiceMessage::GetAllID { feedback }=>{
                let _=feedback.send(self.filter.embeddings.keys().copied().collect());
            }
        }
        Ok(())
    }
    async fn stop(&mut self){
        if let Err(e)=self.endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::SetEmbedAvailable { available: false }).await{
            warn!("通知 MemoryService 本 EmbedService 已停止失败: {}",e)
        };
        if let Ok(bytes) = rkyv::to_bytes::<rkyv::rancor::Error>(&self.filter).context("序列化 rkyv 失败"){
            if let Err(e)= self.endpoint.send(FS_SERVICE, FsServiceMessage::WriteBytes { path: self.filter_path.clone(), data: bytes.into_vec() }).await {
                warn!("保存 filter 失败: {}",e)
            };
        };
        
    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

#[derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, Debug, PartialEq)]
pub struct EmbeddingFilter {
    pub embeddings:HashMap<i64,Embedding>,
}

impl EmbeddingFilter {
    fn new()->Self{
        Self { embeddings: HashMap::new() }
    }
}

impl FilterT for EmbeddingFilter {
    fn hnsw_filter(&self, id: &DataId) -> bool {
        self.embeddings.contains_key(&(*id as i64))
    }
}

impl EmbedService {
    async fn embed(&self,messages:Vec<String>)->Result<Vec<Embedding>> {
        self.embed_model.embed(self.dimensions, messages).await
    }
}

#[derive(Debug,Deserialize)]
struct Config {
    base_url: String,
    model: String,
    api_key_env_var: String,
    dimensions: u32,
}