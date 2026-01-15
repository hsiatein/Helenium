use std::time::Duration;

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
use tokio::time::timeout;

use crate::ApiConfig;
use crate::RequiredTools;
use crate::ToolIntent;
use crate::memory::ChatRole;
use crate::memory::MemoryContent;
use crate::memory::MemoryEntry;

#[heleny_macros::chat_model]
#[derive(Debug, Clone)]
pub struct PlannerModel {
    preset: String,
    model: String,
    client: Client<OpenAIConfig>,
    schema: &'static str,
    timeout: u64,
}

impl PlannerModel {
    pub fn new(preset: String, api_config: ApiConfig, timeout: u64) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_config.base_url)
            .with_api_key(api_config.api_key);
        Self {
            preset,
            model: api_config.model,
            client: Client::with_config(config),
            schema: PLANNER_SCHEMA,
            timeout,
        }
    }

    pub async fn get_tools_list(&self, message: &str) -> Result<RequiredTools> {
        let entry = MemoryEntry::new(ChatRole::User, MemoryContent::Text(message.to_string()));
        let message = entry.to_chat_message()?;
        let response = self._chat(vec![message]).await?;
        serde_json::from_str(&response).context(format!(
            "解析 Planner 回复为 RequiredTools 失败, 回复内容: {}",
            response
        ))
    }
}

#[heleny_macros::chat_model]
#[derive(Debug)]
pub struct ExecutorModel {
    preset: String,
    model: String,
    client: Client<OpenAIConfig>,
    schema: &'static str,
    memory: Vec<ChatCompletionRequestMessage>,
    timeout: u64,
}

impl ExecutorModel {
    pub fn new(preset: String, api_config: ApiConfig, timeout: u64) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(api_config.base_url)
            .with_api_key(api_config.api_key);
        Self {
            preset,
            model: api_config.model,
            client: Client::with_config(config),
            schema: EXECUTOR_SCHEMA,
            memory: Vec::new(),
            timeout,
        }
    }

    pub fn add_preset(&mut self, append: &str) {
        self.preset.push_str(append);
    }

    pub async fn get_intent<T: Into<String>>(&mut self, message: T) -> Result<ToolIntent> {
        let checkpoint = self.memory.len();
        let intent = self._get_intent(message).await;
        if intent.is_err() {
            self.rollback(checkpoint);
        }
        intent
    }

    async fn _get_intent<T: Into<String>>(&mut self, message: T) -> Result<ToolIntent> {
        let message = MemoryEntry::new(ChatRole::System, MemoryContent::Text(message.into()))
            .to_chat_message()?;
        self.memory.push(message);
        let response = self._chat(self.memory.to_owned()).await?;
        let intent = serde_json::from_str(&response).context(format!(
            "解析 Executor 回复为 ToolIntent 失败, 回复内容: {}",
            response
        ))?;
        let message = MemoryEntry::new(ChatRole::Assistant, MemoryContent::Text(response))
            .to_chat_message()?;
        self.memory.push(message);
        Ok(intent)
    }

    fn rollback(&mut self, checkpoint: usize) {
        while self.memory.len() > checkpoint {
            self.memory.pop();
        }
    }
}

#[async_trait]
pub trait ChatModel {
    fn schema(&self) -> &'static str;
    fn client(&self) -> &Client<OpenAIConfig>;
    fn model(&self) -> String;
    fn preset(&self) -> String;
    fn timeout_secs(&self) -> u64;
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
        
        let response = timeout(Duration::from_secs(self.timeout_secs()), self
            .client()
            .chat()
            .create(request))
            .await
            .context("获取回复超时")?
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

pub static HELENY_SCHEMA: &'static str = r#"{
  "type": "object",
  "properties": {
    "content": {
      "type": "string",
      "description": "包含动作描述和回复文本的字符串，格式为：（动作）回复内容"
    },
    "need_help": {
      "oneOf": [
        { "type": "null" },
        { 
          "type": "string", 
          "description": "对用户需求的总结。如果需要外部组件（如 Planner 或 Executor）协助，则提供总结；否则为 null。" 
        }
      ]
    }
  },
  "required": ["content", "need_help"],
  "additionalProperties": false
}"#;

pub static PLANNER_SCHEMA: &'static str = r#"{
  "type": "object",
  "properties": {
    "reason": {
      "type": "string",
      "description": "详细解释判断用户需求并选择对应工具的逻辑和原因。"
    },
    "tools": {
      "description": "需要调用的工具名称列表；如果无论用什么工具都无法完成要求，则为null; 如果不需要任何工具即可完成要求，则为[]。",
      "oneOf": [
        {
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "required": ["reason", "tools"],
  "additionalProperties": false
}"#;

pub static EXECUTOR_SCHEMA: &'static str = r#"{
  "type": "object",
  "properties": {
    "reason": {
      "type": "string",
      "description": "解释你为何选择该工具与该命令，用于内部调试和审计。"
    },
    "tool": {
      "description": "需要调用的工具名称；如果不需要调用工具，则为 null。",
      "oneOf": [
        { "type": "string" },
        { "type": "null" }
      ]
    },
    "command": {
      "description": "需要执行的工具命令；如果不需要执行命令，则为 null。",
      "oneOf": [
        { "type": "string" },
        { "type": "null" }
      ]
    },
    "args": {
      "type": "object",
      "description": "给命令的参数，键为参数名，值为参数值",
      "additionalProperties": true,
      "default": {}
    }
  },
  "required": ["reason"],
  "additionalProperties": false
}"#;
