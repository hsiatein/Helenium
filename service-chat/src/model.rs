use anyhow::Context;
use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::ChatCompletionRequestMessage;
use async_openai::types::chat::ChatCompletionRequestSystemMessageArgs;
use async_openai::types::chat::CreateChatCompletionRequestArgs;
use async_openai::types::chat::ResponseFormat;
use async_openai::types::chat::ResponseFormatJsonSchema;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::memory::ChatRole;
use heleny_proto::memory::MemoryContent;
use heleny_proto::memory::MemoryEntry;
use heleny_proto::name::MEMORY_SERVICE;
use heleny_service::MemoryServiceMessage;
use tokio::sync::oneshot;
use crate::HELENY_SCHEMA;
use crate::chat_config::ApiConfig;
use heleny_proto::HelenyReply;

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

impl ChatModel for HelenyModel {
    fn schema(&self) -> &'static str {
        self.schema
    }
    fn client(&self) -> &Client<OpenAIConfig> {
        &self.client
    }
    fn model(&self) -> String {
        self.model.clone()
    }
    fn preset(&self) -> String {
        self.preset.clone()
    }
}

#[async_trait]
pub trait ChatModel {
    fn schema(&self) -> &'static str;
    fn client(&self) -> &Client<OpenAIConfig>;
    fn model(&self) -> String;
    fn preset(&self) -> String;
    async fn _chat(&self, messages: Vec<ChatCompletionRequestMessage>) -> Result<String> {
        let preset = ChatCompletionRequestSystemMessageArgs::default()
            .content(self.preset().clone())
            .build()
            .context("预设提示词失败")?;
        let mut preset_messages = vec![preset.into()];
        preset_messages.extend(messages);
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model())
            .messages(preset_messages)
            .n(1)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    schema: Some(serde_json::from_str(self.schema()).context("解析成 Value 失败")?),
                    description: None,
                    name: "math_reasoning".into(),
                    strict: Some(true),
                },
            })
            .build()
            .context("构造请求失败")?;
        let response = self
            .client()
            .chat()
            .create(request)
            .await
            .context("获取回复失败")?;
        let content = response
            .choices
            .first()
            .context("回复数量为空")?
            .message
            .content
            .to_owned()
            .context("回复内容为空")?;
        Ok(content)
    }
}
