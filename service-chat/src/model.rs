use crate::HELENY_SCHEMA;
use anyhow::Context;
use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::ChatCompletionRequestSystemMessageArgs;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::chat_model;
use heleny_proto::ApiConfig;
use heleny_proto::ChatModel;
use heleny_proto::ChatRole;
use heleny_proto::HelenyReply;
use heleny_proto::MEMORY_SERVICE;
use heleny_proto::MemoryContent;
use heleny_proto::MemoryEntry;
use heleny_service::MemoryServiceMessage;
use heleny_service::get_tool_descriptions;
use tokio::sync::oneshot;

#[chat_model]
pub struct HelenyModel {
    preset: String,
    model: String,
    client: Client<OpenAIConfig>,
    schema: &'static str,
    endpoint: Endpoint,
}

impl HelenyModel {
    pub fn new(preset: String, api_config: ApiConfig, endpoint: Endpoint) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_config.base_url)
            .with_api_key(api_config.api_key);
        Self {
            preset,
            model: api_config.model,
            client: Client::with_config(config),
            schema: HELENY_SCHEMA,
            endpoint,
        }
    }

    /// 发送消息进行聊天, 返回本次是否需要调用工具帮助
    pub async fn chat(&self, message: String) -> Result<Option<String>> {
        // Post 用户消息
        let entry = MemoryEntry::new(ChatRole::User, MemoryContent::Text(message));
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { entry })
            .await?;
        // 构造聊天信息
        let tool_descriptions = ChatCompletionRequestSystemMessageArgs::default()
            .content(get_tool_descriptions(&self.endpoint).await?)
            .build()
            .context("生成工具简介失败")?;
        let mut messages = vec![tool_descriptions.into()];
        let (tx, rx) = oneshot::channel();
        self.endpoint
            .send(
                MEMORY_SERVICE,
                MemoryServiceMessage::GetChatMemories { feedback: tx },
            )
            .await?;
        let history = rx.await.context("获取历史信息失败")?;
        messages.extend(history);
        // 获取响应
        let response = self._chat(messages).await?;
        let heleny_reply: HelenyReply =
            serde_json::from_str(&response).context("解析 Response 为 HelenyReply 失败")?;
        // Post 回复
        let HelenyReply { content, need_help } = heleny_reply;
        let entry = MemoryEntry::new(ChatRole::Assistant, MemoryContent::Text(content));
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { entry })
            .await?;
        Ok(need_help)
    }

    /// 发送任务结果给 Heleny, 由 Heleny 来解释给 User
    pub async fn explain_task_result(&self, log: Vec<String>) -> Result<()> {
        // 构造聊天信息
        let log = format!("<task_log>{:?}</task_log>", log);
        let entry = MemoryEntry::new(ChatRole::System, MemoryContent::Text(log));
        let message = vec![entry.to_chat_message()?];
        // 获取响应
        let response = self._chat(message).await?;
        let heleny_reply: HelenyReply =
            serde_json::from_str(&response).context("解析 Response 为 HelenyReply 失败")?;
        // Post 回复
        let entry = MemoryEntry::new(
            ChatRole::Assistant,
            MemoryContent::Text(heleny_reply.content),
        );
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { entry })
            .await?;
        Ok(())
    }
}
