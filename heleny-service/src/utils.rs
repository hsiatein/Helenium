use std::time::Duration;

use anyhow::{Context, Result};
use heleny_bus::endpoint::Endpoint;
use heleny_proto::{config_service_message::ConfigServiceMessage, name::CONFIG_SERVICE};
use serde::de::DeserializeOwned;
use tokio::{sync::oneshot, time::timeout};

pub async fn get_from_config_service<T: DeserializeOwned>(endpoint: &Endpoint) -> Result<T> {
    let (tx, rx) = oneshot::channel();
    endpoint
        .send(
            CONFIG_SERVICE,
            Box::new(ConfigServiceMessage::Get { sender: tx }),
        )
        .await
        .context("获取 ConfigService 的资源发送失败")?;
    let config = timeout(Duration::from_secs(5), rx)
        .await
        .context("获取 ConfigService 的资源超时")?
        .context("获取 ConfigService 的资源失败")?
        .context("获取 ConfigService 的资源为空")?;
    serde_json::from_value(config).context("获取到 ConfigService 的资源, 但是解析失败")
}
