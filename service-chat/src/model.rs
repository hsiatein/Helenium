use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::Chat;
use heleny_proto::ChatRole;
use heleny_proto::HelenyReply;
use heleny_proto::MEMORY_SERVICE;
use heleny_proto::MemoryContent;
use heleny_proto::MemoryEntry;
use heleny_proto::trim_response;
use heleny_service::MemoryServiceMessage;
use heleny_service::get_tool_descriptions;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::debug;
use tracing::warn;

pub struct HelenyModel {
    preset: MemoryEntry,
    endpoint: Endpoint,
    timeout: Duration,
    chat_model: Box<dyn Chat>,
    rag_num:usize,
}

impl HelenyModel {
    pub fn new(preset: &str, endpoint: Endpoint, timeout:u64, rag_num:usize, chat_model: Box<dyn Chat>) -> Self {
        Self {
            preset:MemoryEntry::temp(ChatRole::System, preset),
            endpoint,
            timeout:Duration::from_secs(timeout),
            rag_num,
            chat_model,
        }
    }

    /// 发送消息进行聊天, 返回本次是否需要调用工具帮助
    pub async fn chat(&self, message: String) -> Result<Option<String>> {
        // Post 用户消息
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role:ChatRole::User,content:message.as_str().into() })
            .await?;
        // 构造聊天信息
        let tool_descriptions = MemoryEntry::temp(ChatRole::System, get_tool_descriptions(&self.endpoint).await?);
        let mut messages: Vec<&MemoryEntry> = vec![&self.preset,&tool_descriptions];
        // rag 检索获取长期记忆
        let mut rag_messages=None;
        if self.rag_num>0 {
            let (tx, rx) = oneshot::channel();
            if let Err(e) =self.endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::GetSimilarMemoryEntries { content: message, num: self.rag_num, feedback: tx }).await {
                warn!("发送获取相似记忆失败: {}",e);
            };
            if let Ok(msgs)=rx.await {
                rag_messages=Some(msgs.into_iter().filter_map(|mut entry| {
                    match &mut entry.content {
                        MemoryContent::Text(msg)=>{
                            *msg=format!("<Memory>{msg}</Memory>");
                            Some(entry)
                        }
                        MemoryContent::Image(_)=>None,
                        MemoryContent::File(_)=>None,
                    }
                }).collect::<Vec<_>>());
            };
        }
        if let Some(rag_messages) = &rag_messages {
            messages.extend(rag_messages.iter().collect::<Vec<_>>());
            debug!("本次聊天长期记忆消息: {:?}",rag_messages);
        }
        // 获取短期记忆
        let (tx, rx) = oneshot::channel();
        self.endpoint
            .send(
                MEMORY_SERVICE,
                MemoryServiceMessage::GetMemoryEntries { feedback: tx },
            )
            .await?;
        let history: Vec<MemoryEntry> = rx.await.context("获取历史信息失败")?;
        messages.extend(history.iter().collect::<Vec<_>>());
        let entry =MemoryEntry::temp(ChatRole::System, ".");
        for _ in 0..10 {
            messages.push(&entry);
        }
        // 获取响应
        let response = self.chat_model.chat(&messages).await?;
        let heleny_reply: HelenyReply =
            match serde_json::from_str(trim_response(&response)?) {
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
        let log = MemoryEntry::temp(ChatRole::System, format!("<task_log>{:?}</task_log>", log));        
        let message = vec![&self.preset,&log];
        // 获取响应
        let response = timeout(self.timeout, self.chat_model.chat(&message)).await.context("获取 Heleny 回复超时")?.context("获取 Heleny 回复失败")?;
        let heleny_reply: HelenyReply =
            serde_json::from_str(trim_response(&response)?).context("解析 Response 为 HelenyReply 失败")?;
        // Post 回复
        self.endpoint
            .send(MEMORY_SERVICE, MemoryServiceMessage::Post { role:ChatRole::Assistant,content:heleny_reply.content.into() })
            .await?;
        Ok(())
    }
}
