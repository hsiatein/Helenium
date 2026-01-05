use anyhow::Context;
use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::chat_model;
use heleny_proto::ChatModel;
use heleny_proto::memory::ChatRole;
use heleny_proto::memory::MemoryContent;
use heleny_proto::memory::MemoryEntry;
use heleny_proto::name::MEMORY_SERVICE;
use heleny_service::MemoryServiceMessage;
use tokio::sync::oneshot;
use crate::HELENY_SCHEMA;
use heleny_proto::ApiConfig;
use heleny_proto::HelenyReply;

#[chat_model]
pub struct HelenyModel {
    preset: String,
    model: String,
    client: Client<OpenAIConfig>,
    schema: &'static str,
    endpoint: Endpoint,
}

impl HelenyModel {
    pub fn new(preset: String, api_config: ApiConfig, endpoint:Endpoint) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_config.base_url)
            .with_api_key(api_config.api_key);
        Self {
            preset,
            model: api_config.model,
            client: Client::with_config(config),
            schema: HELENY_SCHEMA,
            endpoint
        }
    }

    /// 发送消息进行聊天, 返回本次是否需要调用工具帮助
    pub async fn chat(&self, message: String) -> Result<Option<String>> {
        // Post 用户消息
        let entry=MemoryEntry::new(ChatRole::User, MemoryContent::Text(message.clone()));
        let message=entry.to_chat_message()?;
        self.endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::Post { entry }).await?;
        // 构造聊天信息
        let (tx,rx)=oneshot::channel();
        self.endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::GetChatMemories { feedback: tx }).await?;
        let mut history=rx.await.context("获取历史信息失败")?;
        history.push(message);
        // 获取响应
        let response = self._chat(history).await?;
        let heleny_reply:HelenyReply=serde_json::from_str(&response).context("解析 Response 为 HelenyReply 失败")?;
        // Post 回复
        let HelenyReply { content, need_help }=heleny_reply;
        let entry=MemoryEntry::new(ChatRole::Assistant, MemoryContent::Text(content));
        self.endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::Post { entry }).await?;
        Ok(need_help)
    }
}
