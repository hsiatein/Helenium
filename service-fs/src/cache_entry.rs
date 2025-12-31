use anyhow::Context;
use anyhow::Result;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::read_to_string;
use tokio::fs::{self};

#[derive(Clone)]
pub struct CacheEntry {
    pub content: String,
    pub last_modified: SystemTime,
}

impl CacheEntry {
    pub async fn read(path: &PathBuf) -> Result<Self> {
        let content = read_to_string(path).await.context("读取文件失败")?;
        let last_modified = fs::metadata(path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        Ok(Self {
            content,
            last_modified,
        })
    }
}
