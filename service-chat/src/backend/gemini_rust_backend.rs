use std::fmt::Debug;
use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use heleny_proto::ChatRole;
use gemini_rust::Gemini;
use tracing::info;
use crate::ApiConfig;
use heleny_proto::Chat;
use heleny_proto::MemoryEntry;

#[derive(Debug)]
pub struct GeminiChat {
    api_config: ApiConfig,
}

impl GeminiChat {
    pub fn new(api_config: ApiConfig) -> Self {
        Self {
            api_config,
        }
    }
}

#[async_trait]
impl Chat for GeminiChat {
    async fn chat(&self,messages: &[&MemoryEntry])->Result<String> {
        let ApiConfig { base_url:_, model, api_key_env_var:_, api_key }= self.api_config.clone();
        info!("当前聊天模型 {}",model);
        let model = if model.starts_with("models/") {
            model
        } else {
            format!("models/{}", model)
        };
        let client = Gemini::with_model(api_key, model)?;
        let mut conversation_builder = client
        .generate_content();
        for msg in messages {
            match msg.role {
                ChatRole::System=>{
                    conversation_builder=conversation_builder.with_user_message(entry_to_string(*msg))
                }
                ChatRole::Assistant=>{
                    conversation_builder=conversation_builder.with_model_message(entry_to_string(*msg))
                }
                ChatRole::User=>{
                    conversation_builder=conversation_builder.with_user_message(entry_to_string(*msg))
                }
            }
        }
        let resp=conversation_builder.execute().await?;
        let text = resp.text();
        if text.trim().is_empty() {
            return Err(anyhow!(
                "Gemini 返回空响应: prompt_feedback={:?}",
                resp.prompt_feedback
            ));
        }
        Ok(text)
    }
}

fn entry_to_string(value: &MemoryEntry) -> String {
    let content = value.time.to_string() + ":" + value.content.to_str();
    content
}

