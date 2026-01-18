mod async_openai_backend;
mod gemini_rust_backend;

use anyhow::Result;
use async_openai_backend::AsyncOpenaiChat;
use genai::{Client, adapter::AdapterKind};
use heleny_proto::{Chat, Embed};

use crate::{backend::{async_openai_backend::AsyncOpenaiEmbed, gemini_rust_backend::GeminiChat}, config::ApiConfig};

pub async fn get_chat_model(api_config:ApiConfig,schema:&'static str)->Result<Box<dyn Chat>> {
    let client = Client::default();
    let adapter_kind = client.resolve_service_target(&api_config.model).await?.model.adapter_kind;
    match adapter_kind {
        AdapterKind::Gemini=>{
            Ok(Box::new(GeminiChat::new(api_config)) as Box<dyn Chat>)
        }
        _=>{
            Ok(Box::new(AsyncOpenaiChat::new(api_config, schema)) as Box<dyn Chat>)
        }
    }
}

pub fn get_embed_model(base_url:String,model:String,api_key:String)->Result<Box<dyn Embed>> {
    let client=AsyncOpenaiEmbed::new(base_url, model, api_key);
    Ok(Box::new(client))
}