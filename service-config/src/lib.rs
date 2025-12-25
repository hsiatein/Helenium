use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::{
    config_service_message::ConfigServiceMessage,
    message::{AnyMessage, downcast},
    role::ServiceRole,
};
use heleny_service::Service;
use serde_json::{Map, Value};
use std::{path::PathBuf, str::FromStr};
use tokio::{fs, task::JoinHandle, time::Instant};
use tracing::{debug, info, warn};

#[base_service(deps=[])]
pub struct ConfigService {
    endpoint: Endpoint,
    config_path: PathBuf,
    config_value: Map<String, Value>,
    last_edit: Option<DateTime<Local>>,
    save_after: f64,
    is_writing: Option<JoinHandle<Result<PathBuf>>>,
    is_reading: Option<JoinHandle<Result<Map<String, Value>>>>,
}

#[async_trait]
impl Service for ConfigService {
    type MessageType = ConfigServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config_path = match std::env::var("HELENIUM_CONFIG") {
            Ok(path) => PathBuf::from(path),
            Err(e) => {
                warn!(
                    "没有设置 HELENIUM_CONFIG 环境变量: {}, 尝试寻找./Config.toml",
                    e
                );
                match std::env::current_dir() {
                    Ok(path) => path.join("Config.json"),
                    Err(e) => return Err(anyhow::anyhow!("获取当前目录失败: {}", e)),
                }
            }
        };
        let config_string = match fs::read_to_string(&config_path).await {
            Ok(string) => string,
            Err(e) => return Err(anyhow::anyhow!("读取配置文件失败: {}", e)),
        };
        let config_value = serde_json::Value::from_str(&config_string)?;
        let config_value = config_value.as_object().context("无法作为 obj")?.clone();
        debug!("config_value: {:?}", config_value);
        let save_after = config_value
            .get(Self::name())
            .context("读取 ConfigService 字段失败")?
            .get("save_after")
            .context("读取 save_after 字段失败")?
            .as_f64()
            .context("save_after 字段值不是浮点数")?;

        Ok(Box::new(Self {
            endpoint,
            config_path,
            config_value,
            last_edit: None,
            save_after,
            is_writing: None,
            is_reading: None,
        }))
    }
    async fn handle(
        &mut self,
        name: &'static str,
        _role: ServiceRole,
        msg: Box<Self::MessageType>,
    ) -> Result<()> {
        match *msg {
            ConfigServiceMessage::Get { sender } => {
                let _ = sender.send(self.config_value.get(name).cloned());
                Ok(())
            }
            ConfigServiceMessage::Set { value } => {
                self.config_value.insert(name.to_string(), value);
                self.last_edit = Some(Local::now());
                Ok(())
            }
            ConfigServiceMessage::Update => self.update().await,
            ConfigServiceMessage::Persist => self.persist().await,
        }
    }
    async fn stop(&mut self) {
        info!("{} 正在停止...", Self::name());
    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()> {
        let work_status = downcast(msg)?;
        match work_status {
            WorkerMessage::WriteOver => self.post_write().await,
            WorkerMessage::ReadOver => self.post_read().await,
        }
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        if let Some(last_edit) = self.last_edit.take() {
            let dt = Local::now() - last_edit;
            if dt.as_seconds_f64() > self.save_after {
                return self.persist().await;
            } else {
                self.last_edit = Some(last_edit)
            }
        }
        Ok(())
    }
}

impl ConfigService {
    async fn persist(&mut self) -> Result<()> {
        match self.is_writing.take() {
            Some(is_writing) => {
                let finish = is_writing.is_finished();
                self.is_writing = Some(is_writing);
                match finish {
                    true => self.post_write().await,
                    false => Err(anyhow::anyhow!("文件写入未完成")),
                }
            }
            None => {
                let sub = self.endpoint.get_sub_endpoint();
                let value =
                    serde_json::to_string_pretty(&self.config_value).context("配置转字符串错误")?;
                let tmp_path = self
                    .config_path
                    .to_str()
                    .context("转化路径成字符串错误")?
                    .to_string()
                    + ".tmp";
                let handle: tokio::task::JoinHandle<Result<PathBuf>> = tokio::spawn(async move {
                    let result = fs::write(&tmp_path, value)
                        .await
                        .context("写入配置文件时出错");
                    let _ = sub.send(Box::new(WorkerMessage::WriteOver)).await;
                    match result {
                        Ok(_) => Ok(PathBuf::from(tmp_path)),
                        Err(e) => Err(e),
                    }
                });
                self.is_writing = Some(handle);
                Ok(())
            }
        }
    }

    async fn post_write(&mut self) -> Result<()> {
        let tmp_path = self
            .is_writing
            .take()
            .context("未找到写入任务句柄")?
            .await
            .context("获取写入结果失败")?
            .context("写入配置文件失败")?;
        fs::rename(tmp_path, &self.config_path)
            .await
            .context("替换配置文件失败")
    }

    async fn post_read(&mut self) -> Result<()> {
        let new_value = self
            .is_reading
            .take()
            .context("未找到读取任务句柄")?
            .await
            .context("获取读取结果失败")?
            .context("读取配置文件失败")?;
        self.config_value = new_value;
        Ok(())
    }

    async fn update(&mut self) -> Result<()> {
        match self.is_reading.take() {
            Some(is_reading) => {
                let finish = is_reading.is_finished();
                self.is_reading = Some(is_reading);
                match finish {
                    true => self.post_read().await,
                    false => Err(anyhow::anyhow!("文件读取未完成")),
                }
            }
            None => {
                let sub = self.endpoint.get_sub_endpoint();
                let path = self.config_path.clone();
                let handle = tokio::spawn(async move {
                    let str = match fs::read_to_string(path).await.context("读取配置文件时出错")
                    {
                        Ok(str) => str,
                        Err(e) => {
                            let _ = sub.send(Box::new(WorkerMessage::ReadOver)).await;
                            return Err(e);
                        }
                    };
                    let config_value = match Value::from_str(str.as_str())
                        .context("解析配置文件失败")?
                        .as_object()
                        .context("无法转为 obj")
                    {
                        Ok(str) => str.clone(),
                        Err(e) => {
                            let _ = sub.send(Box::new(WorkerMessage::ReadOver)).await;
                            return Err(e);
                        }
                    };
                    let _ = sub.send(Box::new(WorkerMessage::ReadOver)).await;
                    Ok(config_value)
                });
                self.is_reading = Some(handle);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
enum WorkerMessage {
    WriteOver,
    ReadOver,
}

#[cfg(test)]
mod tests {}
