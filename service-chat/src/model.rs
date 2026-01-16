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
use heleny_proto::build_async_openai_msg;
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
    timeout: u64,
}

impl HelenyModel {
    pub fn new(preset: String, api_config: ApiConfig, endpoint: Endpoint, timeout:u64) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_config.base_url)
            .with_api_key(api_config.api_key);
        Self {
            preset,
            model: api_config.model,
            client: Client::with_config(config),
            schema: HELENY_SCHEMA,
            endpoint,
            timeout,
        }
    }

    /// 发送消息进行聊天, 返回本次是否需要调用工具帮助
    pub async fn chat(&self, message: String) -> Result<Option<String>> {
        // Post 用户消息
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role:ChatRole::User,content:message.into() })
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
        let entry =build_async_openai_msg(ChatRole::System, ".")?;
        for _ in 0..10 {
            messages.push(entry.clone());
        }
        // 获取响应
        let response = self._chat(messages).await?;
        let heleny_reply: HelenyReply =
            match serde_json::from_str(&response) {
                Ok(resp)=>resp,
                Err(e)=>{
                    return Err(anyhow::anyhow!("解析 {} 为 HelenyReply失败: {}",response,e));
                }
            };
        // Post 回复
        let HelenyReply { content, need_help } = heleny_reply;
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role:ChatRole::Assistant, content:content.into() })
            .await?;
        Ok(need_help)
    }

    /// 发送任务结果给 Heleny, 由 Heleny 来解释给 User
    pub async fn explain_task_result(&self, log: Vec<String>) -> Result<()> {
        // 构造聊天信息
        let log = format!("<task_log>{:?}</task_log>", log);        
        let message = vec![build_async_openai_msg(ChatRole::System, &log)?];
        // 获取响应
        let response = self._chat(message).await?;
        let heleny_reply: HelenyReply =
            serde_json::from_str(&response).context("解析 Response 为 HelenyReply 失败")?;
        // Post 回复
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role:ChatRole::Assistant,content:heleny_reply.content.into() })
            .await?;
        Ok(())
    }
}
