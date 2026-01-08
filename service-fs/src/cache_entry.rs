use anyhow::Context;
use anyhow::Result;
use heleny_proto::HelenyFile;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::read_to_string;
use tokio::fs::{self};

#[derive(Clone)]
pub struct CacheEntry {
    pub content: HelenyFile,
    pub last_modified: SystemTime,
}

impl CacheEntry {
    pub async fn read_text(path: &PathBuf) -> Result<Self> {
        let content = read_to_string(path).await.context("读取文件失败")?;
        let last_modified: SystemTime = fs::metadata(path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        Ok(Self {
            content: HelenyFile::Text(content),
            last_modified,
        })
    }

    pub async fn read_image(path: &PathBuf) -> Result<Self> {
        let content = tokio::fs::read(path).await?;
        let last_modified: SystemTime = fs::metadata(path)
            .await
            .context("获取文件元数据失败")?
            .modified()
            .context("获取文件修改时间失败")?;
        Ok(Self {
            content: HelenyFile::Image(content),
            last_modified,
        })
    }
}
