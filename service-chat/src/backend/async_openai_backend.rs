use std::fmt::Debug;
use anyhow::Context;
use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::ChatCompletionRequestAssistantMessageArgs;
use async_openai::types::chat::ChatCompletionRequestMessage;
use async_openai::types::chat::ChatCompletionRequestSystemMessageArgs;
use async_openai::types::chat::ChatCompletionRequestUserMessageArgs;
use async_openai::types::chat::CreateChatCompletionRequestArgs;
use async_openai::types::chat::ResponseFormat;
use async_openai::types::chat::ResponseFormatJsonSchema;
use async_openai::types::embeddings::CreateEmbeddingRequest;
use async_openai::types::embeddings::EmbeddingInput;
use async_trait::async_trait;
use heleny_proto::ChatRole;

use async_openai::Embeddings;
use heleny_proto::Embed;
use heleny_proto::Embedding;
use tracing::info;
use crate::ApiConfig;
use heleny_proto::Chat;
use heleny_proto::MemoryEntry;

#[derive(Debug)]
pub struct AsyncOpenaiChat {
    client: Client<OpenAIConfig>,
    model: String,
    schema: &'static str
}

impl AsyncOpenaiChat {
    pub fn new(api_config: ApiConfig,schema:&'static str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_config.base_url)
            .with_api_key(api_config.api_key);
        Self {
            client: Client::with_config(config),
            model: api_config.model,
            schema,
        }
    }
}

#[async_trait]
impl Chat for AsyncOpenaiChat {
    async fn chat(&self,messages: &[&MemoryEntry])->Result<String> {
        info!("当前聊天模型 {}",self.model);
        let messages:Vec<_>=messages.iter().filter_map(|&msg| entry_to_async_openai(msg).ok()).collect();
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .n(1)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    schema: Some(serde_json::from_str(self.schema).context("解析成 Value 失败")?),
                    description: None,
                    name: "math_reasoning".into(),
                    strict: Some(true),
                },
            })
            .build()
            .context("构造请求失败")?;
        
        let response = self
            .client
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

fn entry_to_async_openai(value: &MemoryEntry) -> Result<ChatCompletionRequestMessage> {
    let content = value.time.to_string() + ":" + value.content.to_str();
    let msg = match value.role {
        ChatRole::System => ChatCompletionRequestSystemMessageArgs::default()
            .content(content)
            .build()?
            .into(),
        ChatRole::User => ChatCompletionRequestUserMessageArgs::default()
            .content(content)
            .build()?
            .into(),
        ChatRole::Assistant => ChatCompletionRequestAssistantMessageArgs::default()
            .content(content)
            .build()?
            .into(),
    };
    Ok(msg)
}

#[derive(Debug)]
pub struct AsyncOpenaiEmbed {
    client: Client<OpenAIConfig>,
    model: String,
}

impl AsyncOpenaiEmbed {
    pub fn new(base_url:String,model:String,api_key:String)->Self{
        let config = OpenAIConfig::new()
            .with_api_base(base_url).with_api_key(api_key);
        let client = Client::with_config(config);
        Self { client, model }
    }
}

#[async_trait]
impl Embed for AsyncOpenaiEmbed {
    async fn embed(&self,dimensions: u32, messages: Vec<String>)->Result<Vec<Embedding>> {
        let mut request=CreateEmbeddingRequest::default();
        request.model=self.model.clone();
        request.dimensions=Some(dimensions);
        request.input=EmbeddingInput::StringArray(messages);
        let embedding=Embeddings::new(&self.client).create(request).await?;
        let embeddings=embedding.data.into_iter().map(|vec| Embedding::new(vec.embedding)).collect();
        Ok(embeddings)
    }
}