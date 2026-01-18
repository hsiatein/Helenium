use std::path::PathBuf;
use std::time::Duration;

use crate::ConfigServiceMessage;
use crate::FsServiceMessage;
use crate::HubServiceMessage;
use crate::KernelServiceMessage;
use crate::ToolkitServiceMessage;
use anyhow::Context;
use anyhow::Result;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::CONFIG_SERVICE;
use heleny_proto::FS_SERVICE;
use heleny_proto::HUB_SERVICE;
use heleny_proto::HelenyToolFactory;
use heleny_proto::KERNEL_SERVICE;
use heleny_proto::ResourcePayload;
use heleny_proto::TOOLKIT_SERVICE;
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio::time::timeout;

pub async fn get_from_config_service<T: DeserializeOwned>(endpoint: &Endpoint) -> Result<T> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(CONFIG_SERVICE, ConfigServiceMessage::Get { sender: tx })
        .await
        .context("获取 ConfigService 的资源发送失败")?;
    let config = timeout(Duration::from_secs(10), rx)
        .await
        .context("获取 ConfigService 的资源超时")?
        .context("获取 ConfigService 的资源失败")?
        .context("获取 ConfigService 的资源为空")?;
    serde_json::from_value(config).context("获取到 ConfigService 的资源, 但是解析失败")
}

pub async fn import_from_config_service<T: DeserializeOwned,U: Into<String>>(endpoint: &Endpoint,name: U) -> Result<T> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(CONFIG_SERVICE, ConfigServiceMessage::Import { key: name.into(), feedback: tx })
        .await
        .context("导入 ConfigService 的变量发送失败")?;
    let config = timeout(Duration::from_secs(10), rx)
        .await
        .context("导入 ConfigService 的变量超时")?
        .context("导入 ConfigService 的变量失败")?;
    serde_json::from_value(config).context("导入成功 ConfigService 的变量, 但是解析失败")
}

pub async fn update_config_service(endpoint: &Endpoint) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(CONFIG_SERVICE, ConfigServiceMessage::Update { feedback: tx } )
        .await
        .context("发送 Update 失败")?;
    timeout(Duration::from_secs(10), rx)
        .await
        .context("获取 ConfigService 的资源超时")?
        .context("获取 Update 反馈失败")
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

pub async fn list_via_fs_service<T: Into<PathBuf>>(
    endpoint: &Endpoint,
    path: T,
) -> Result<Vec<PathBuf>> {
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

pub async fn wait_for(endpoint: &Endpoint, name: &str) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            KERNEL_SERVICE,
            KernelServiceMessage::WaitFor {
                name: name.to_string(),
                sender: tx,
            },
        )
        .await
        .context("等待服务的消息发送失败")?;
    let data = rx.await.context("获取工具简介失败")?;
    data
}

pub async fn register_tool_factory<T: HelenyToolFactory>(endpoint: &Endpoint, factory: T) {
    let register_endpoint = endpoint.create_sender_endpoint();
    tokio::spawn(async move {
        if wait_for(&register_endpoint, TOOLKIT_SERVICE).await.is_err() {
            return;
        };
        let _ = register_endpoint
            .send(
                TOOLKIT_SERVICE,
                ToolkitServiceMessage::Register {
                    factory: Box::new(factory),
                },
            )
            .await;
    });
}

pub async fn publish_resource<T: Into<String>>(
    endpoint: &Endpoint,
    resource_name: T,
    receiver: watch::Receiver<ResourcePayload>,
) -> Result<()> {
    endpoint
        .send(
            HUB_SERVICE,
            HubServiceMessage::Publish {
                resource_name: resource_name.into(),
                receiver,
            },
        )
        .await
}

pub async fn get_resource<T: Into<String>>(
    endpoint: &Endpoint,
    resource_name: T,
) -> Result<ResourcePayload> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            HUB_SERVICE,
            HubServiceMessage::Get {
                resource_name: resource_name.into(),
                feedback: tx,
            },
        )
        .await?;
    rx.await.context("获取资源失败")
}

pub async fn subscribe_resource<T: Into<String>>(
    endpoint: &Endpoint,
    resource_name: T,
) -> Result<()> {
    endpoint
        .send(
            HUB_SERVICE,
            HubServiceMessage::Subscribe {
                resource_name: resource_name.into(),
            },
        )
        .await
}

pub async fn unsubscribe_resource<T: Into<String>>(
    endpoint: &Endpoint,
    resource_name: T,
) -> Result<()> {
    endpoint
        .send(
            HUB_SERVICE,
            HubServiceMessage::Unsubscribe {
                resource_name: resource_name.into(),
            },
        )
        .await
}
