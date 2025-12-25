mod auth_config;

use std::{collections::HashSet, time::Duration};
use ed25519_dalek::{Signature, VerifyingKey, pkcs8::DecodePublicKey};
use tracing::{debug, warn};
use tokio::{sync::oneshot, time::{Instant, timeout}};
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_proto::{auth_service_message::AuthServiceMessage, config_service_message::ConfigServiceMessage, message::AnyMessage, role::ServiceRole};
use async_trait::async_trait;
use anyhow::{Context, Result};

use crate::auth_config::AuthConfig;



#[base_service(deps=["ConfigService"])]
pub struct AuthService{
    endpoint:Endpoint,
    pub_keys:Vec<VerifyingKey>,
    challenges:HashSet<[u8;32]>,
}

#[async_trait]
impl Service for AuthService {
    type MessageType= AuthServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let (tx,rx)=oneshot::channel();
        endpoint.send("ConfigService", Box::new(ConfigServiceMessage::Get { sender: tx })).await.context("AuthService 获取 ConfigService 的资源发送失败")?;
        let config=timeout(Duration::from_secs(5), rx).await.context("获取 ConfigService 的资源超时")?.context("获取 ConfigService 的资源失败")?.context("获取 ConfigService 的资源为空")?;
        let config:AuthConfig=serde_json::from_str(&config.to_string())?;
        debug!("AuthService Config: {:?}",config);
        let pub_keys=config.pub_keys.iter().cloned().filter_map(|pub_key|{
            match VerifyingKey::from_public_key_pem(&pub_key){
                Ok(pub_key)=>Some(pub_key),
                Err(e)=>{
                    warn!("{:?}解析失败, 忽略: {}",pub_key,e);
                    None
                }
            }
        }).collect();
        debug!("AuthService 公钥: {:?}",pub_keys);

        let instance=Self {
            endpoint,
            pub_keys,
            challenges:HashSet::new(),
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: &'static str,
        _role: ServiceRole,
        msg: Box<Self::MessageType>,
    ) -> Result<()>{
        match *msg {
            AuthServiceMessage::GetChallenge{msg_sender}=>{
                let challenge=generate_challenge();
                let _=msg_sender.send(challenge);
                self.challenges.insert(challenge);
            }
            AuthServiceMessage::Verify { msg, signature, pass }=>{
                if self.challenges.remove(&msg) {
                    let _=pass.send(self.verify(msg, signature));
                }
            }
        }
        Ok(())
    }
    async fn stop(&mut self){

    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    
}

use rand::rngs::OsRng;
use rand::RngCore;

fn generate_challenge() -> [u8;32] {
    let mut challenge = [0u8; 32];
    OsRng.fill_bytes(&mut challenge);
    challenge
}

impl AuthService {
    fn verify(&mut self,msg:[u8;32],signature:Signature)->bool{
        self.pub_keys.iter_mut().any(|key| {
            key.verify_strict(&msg, &signature).is_ok()
        })
    }
}

#[cfg(test)]
mod tests;