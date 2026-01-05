use std::path::PathBuf;
use std::time::Duration;

use crate::ConfigServiceMessage;
use crate::FsServiceMessage;
use crate::ToolkitServiceMessage;
use anyhow::Context;
use anyhow::Result;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::name::CONFIG_SERVICE;
use heleny_proto::name::FS_SERVICE;
use heleny_proto::name::TOOLKIT_SERVICE;
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;
use tokio::time::timeout;

pub async fn get_from_config_service<T: DeserializeOwned>(endpoint: &Endpoint) -> Result<T> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(CONFIG_SERVICE, ConfigServiceMessage::Get { sender: tx })
        .await
        .context("获取 ConfigService 的资源发送失败")?;
    let config = timeout(Duration::from_secs(5), rx)
        .await
        .context("获取 ConfigService 的资源超时")?
        .context("获取 ConfigService 的资源失败")?
        .context("获取 ConfigService 的资源为空")?;
    serde_json::from_value(config).context("获取到 ConfigService 的资源, 但是解析失败")
}

pub async fn read_via_fs_service<T: Into<PathBuf>>(endpoint: &Endpoint, path: T) -> Result<String> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            FS_SERVICE,
            FsServiceMessage::Read {
                path: path.into(),
                feedback: tx,
            },
        )
        .await
        .context("获取 FsService 文件的消息发送失败")?;
    let data = rx.await.context("获取 FsService 的文件失败")?;
    Ok(data)
}

pub async fn list_via_fs_service<T: Into<PathBuf>>(endpoint: &Endpoint, path: T) -> Result<Vec<PathBuf>> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            FS_SERVICE,
            FsServiceMessage::List {
                dir: path.into(),
                feedback: tx,
            },
        )
        .await
        .context("获取 FsService 文件的消息发送失败")?;
    let data = rx.await.context("读取目录失败")?;
    Ok(data)
}

pub async fn get_tool_descriptions(endpoint: &Endpoint) -> Result<String> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            TOOLKIT_SERVICE,
            ToolkitServiceMessage::GetIntro { feedback: tx },
        )
        .await
        .context("获取工具简介的消息发送失败")?;
    let data = rx.await.context("获取工具简介失败")?;
    Ok(data)
}
