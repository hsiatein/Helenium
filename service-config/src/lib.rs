use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::{config_service_message::ConfigServiceMessage, role::ServiceRole};
use heleny_service::Service;
use std::path::PathBuf;
use tracing::{info, warn};

#[base_service(deps=[])]
pub struct ConfigService {
    endpoint: Endpoint,
    config_path: PathBuf,
    config_value: toml::Value,
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
                    Ok(path) => path.join("Config.toml"),
                    Err(e) => return Err(anyhow::anyhow!("获取当前目录失败: {}", e)),
                }
            }
        };
        let config_string = match std::fs::read_to_string(&config_path) {
            Ok(string) => string,
            Err(e) => return Err(anyhow::anyhow!("读取配置文件失败: {}", e)),
        };
        let config_value = match toml::Value::try_from(config_string) {
            Ok(value) => value,
            Err(e) => return Err(anyhow::anyhow!("解析配置文件失败: {}", e)),
        };

        Ok(Box::new(Self {
            endpoint,
            config_path,
            config_value,
        }))
    }
    async fn handle(
        &mut self,
        name: &'static str,
        role: ServiceRole,
        msg: Box<Self::MessageType>,
    ) -> Result<()> {
        match *msg {
            ConfigServiceMessage::Get { path, sender } => {}
            ConfigServiceMessage::Set { path, value } => {}
            ConfigServiceMessage::Update => {}
            ConfigServiceMessage::Persist => {}
        }
        Ok(())
    }
    async fn stop(&mut self) {
        info!("{} 正在停止...", Self::name());
    }
}

impl ConfigService {}
