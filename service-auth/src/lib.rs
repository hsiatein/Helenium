mod config;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use ed25519_dalek::pkcs8::DecodePublicKey;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::CONFIG_SERVICE;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::downcast;
use heleny_service::AuthServiceMessage;
use heleny_service::ConfigServiceMessage;
use heleny_service::Service;
use heleny_service::get_from_config_service;
use rand::RngCore;
use rand::rngs::OsRng;
use serde_json::Value;
use std::collections::HashSet;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tracing::debug;
use tracing::warn;

use crate::config::AuthConfig;

#[base_service(deps=["ConfigService"])]
pub struct AuthService {
    endpoint: Endpoint,
    pub_keys: Vec<VerifyingKey>,
    challenges: HashSet<[u8; 32]>,
    is_updating: Option<JoinHandle<Result<Vec<VerifyingKey>>>>,
}

#[derive(Debug)]
enum WorkerMessage {
    UpdateOver,
}

#[async_trait]
impl Service for AuthService {
    type MessageType = AuthServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config = get_from_config_service::<AuthConfig>(&endpoint).await?;
        debug!("AuthService Config: {:?}", config);
        let pub_keys = config
            .pub_keys
            .iter()
            .cloned()
            .filter_map(
                |pub_key| match VerifyingKey::from_public_key_pem(&pub_key) {
                    Ok(pub_key) => Some(pub_key),
                    Err(e) => {
                        warn!("{:?}解析失败, 忽略: {}", pub_key, e);
                        None
                    }
                },
            )
            .collect();
        // debug!("AuthService 公钥: {:?}", pub_keys);

        let instance = Self {
            endpoint,
            pub_keys,
            challenges: HashSet::new(),
            is_updating: None,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: AuthServiceMessage,
    ) -> Result<()> {
        match msg {
            AuthServiceMessage::GetChallenge { msg_sender } => {
                let challenge = generate_challenge();
                let _ = msg_sender.send(challenge);
                self.challenges.insert(challenge);
            }
            AuthServiceMessage::Verify {
                msg,
                signature,
                pass,
            } => {
                if self.challenges.remove(&msg) {
                    let _ = pass.send(self.verify(msg, signature));
                }
            }
            AuthServiceMessage::Update => {
                return self.handle_update().await;
            }
        }
        Ok(())
    }
    async fn stop(&mut self) {
        if let Some(handle) = self.is_updating.take() {
            handle.abort();
        }
    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()> {
        let msg = downcast::<WorkerMessage>(msg)?;
        match msg {
            WorkerMessage::UpdateOver => self.post_update().await,
        }
    }
    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

fn generate_challenge() -> [u8; 32] {
    let mut challenge = [0u8; 32];
    OsRng.fill_bytes(&mut challenge);
    challenge
}

impl AuthService {
    fn verify(&mut self, msg: [u8; 32], signature: Signature) -> bool {
        self.pub_keys
            .iter_mut()
            .any(|key| key.verify_strict(&msg, &signature).is_ok())
    }
    async fn handle_update(&mut self) -> Result<()> {
        if let Some(is_updating) = &self.is_updating {
            if is_updating.is_finished() {
                self.post_update().await?;
            }
        }
        let sub_endpoint = self.endpoint.create_sub_endpoint()?;
        let (tx, rx) = oneshot::channel();
        self.endpoint
            .send(CONFIG_SERVICE, ConfigServiceMessage::Get { sender: tx })
            .await
            .context("AuthService 获取 ConfigService 的资源发送失败")?;
        let handle = tokio::spawn(async move {
            let result = worker_update(rx).await;
            let _ = sub_endpoint.send(Box::new(WorkerMessage::UpdateOver)).await;
            result
        });
        self.is_updating = Some(handle);
        Ok(())
    }

    async fn post_update(&mut self) -> Result<()> {
        let keys = self
            .is_updating
            .take()
            .context("没有正在更新的子任务")?
            .await
            .context("获取await结果失败")?
            .context("未得到最新 VerifyingKeys")?;
        self.pub_keys = keys;
        Ok(())
    }
}

async fn worker_update(rx: oneshot::Receiver<Option<Value>>) -> Result<Vec<VerifyingKey>> {
    let new_value = rx.await.context("获取回应失败")?.context("获取资源失败")?;
    let config: AuthConfig = serde_json::from_str(&new_value.to_string())?;
    let pub_keys = config
        .pub_keys
        .iter()
        .cloned()
        .filter_map(
            |pub_key| match VerifyingKey::from_public_key_pem(&pub_key) {
                Ok(pub_key) => Some(pub_key),
                Err(e) => {
                    warn!("{:?}解析失败, 忽略: {}", pub_key, e);
                    None
                }
            },
        )
        .collect();
    Ok(pub_keys)
}

#[cfg(test)]
mod tests;
