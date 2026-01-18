use std::fmt::Debug;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use rkyv::Archive;
use rkyv::Deserialize;
use rkyv::Serialize;
use tokio::time::timeout;

use crate::MemoryEntry;
use crate::RequiredTools;
use crate::ToolIntent;
use crate::memory::ChatRole;

#[derive(Debug)]
pub struct PlannerModel {
    preset: MemoryEntry,
    timeout: Duration,
    chat_model: Box<dyn Chat>
}

pub fn trim_response(response: &String)->Result<&str> {
    let start = match response.find('{') {
        Some(index)=>index,
        None=> {
            return Err(anyhow!("没有 {{ : {response}"));
        }
    };
    let end = match response.rfind('}') {
        Some(index)=>index,
        None=> {
            return Err(anyhow!("没有 }} : {response}"));
        }
    };
    if start >= end {
        return Err(anyhow::anyhow!("{{, }}位置不合法"));
    }
    Ok(&response[start..=end])
}

impl PlannerModel {
    pub fn new(preset: String, timeout: u64,chat_model: Box<dyn Chat>) -> Self {
        Self {
            preset:MemoryEntry::temp(ChatRole::System, preset),
            timeout:Duration::from_secs(timeout),
            chat_model,
        }
    }

    pub async fn get_tools_list(&self, message: &str) -> Result<RequiredTools> {
        let entry=MemoryEntry::temp( ChatRole::User, message);
        let response=match timeout(self.timeout, self.chat_model.chat(&[&self.preset,&entry])).await.context("获取 tools_list 超时")?{
            Ok(resp)=>resp,
            Err(e)=> return Err(anyhow!("获取 response 失败: {e}"))
        };
        serde_json::from_str(trim_response(&response)?).context(format!(
            "解析 Planner 回复为 RequiredTools 失败, 回复内容: {}",
            response
        ))
    }
}

#[derive(Debug)]
pub struct ExecutorModel {
    memory: Vec<MemoryEntry>,
    timeout: Duration,
    chat_model: Box<dyn Chat>
}

impl ExecutorModel {
    pub fn new(preset: &str, timeout: u64,chat_model: Box<dyn Chat>) -> Self {
        Self {
            memory: vec![MemoryEntry::temp(ChatRole::System, preset)],
            timeout:Duration::from_secs(timeout),
            chat_model,
        }
    }

    pub fn add_preset(&mut self, append: &str) {
        self.memory.push(MemoryEntry::temp(ChatRole::System, append));
    }

    pub async fn get_intent(&mut self, message: &str) -> Result<ToolIntent> {
        let checkpoint = self.memory.len();
        let intent = self._get_intent(message).await;
        if intent.is_err() {
            self.rollback(checkpoint);
        }
        intent
    }

    async fn _get_intent(&mut self, message: &str) -> Result<ToolIntent> {
        let role= if self.memory.len() <3 {
            ChatRole::User
        }else {
            ChatRole::System
        };
        let message = MemoryEntry::temp(role,message);
        self.memory.push(message);
        let messages=self.memory.iter().collect::<Vec<_>>();
        let response = match timeout(self.timeout, self.chat_model.chat(&messages)).await.context("获取 tools_intent 超时")? {
            Ok(resp)=>resp,
            Err(e)=> return Err(anyhow!("获取 tools_intent 失败: {e}"))
        };
        let intent = serde_json::from_str(trim_response(&response)?).context(format!(
            "解析 Executor 回复为 ToolIntent 失败, 回复内容: {}",
            response
        ))?;
        let message = MemoryEntry::temp(ChatRole::Assistant,response);
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
pub trait Chat:Debug+Sync+Send {
    async fn chat(&self,messages: &[&MemoryEntry])->Result<String>;
}

#[async_trait]
pub trait Embed:Debug+Sync+Send {
    async fn embed(&self,dimensions: u32, messages: Vec<String>)->Result<Vec<Embedding>>;
}

#[derive(Archive, Serialize, Deserialize, Debug, PartialEq)]
pub struct Embedding {
    pub vector: Vec<f32>,
}

impl Embedding {
    pub fn new(vec:Vec<f32>) -> Self {
        let mut ins=Self {vector: vec};
        ins.normalize();
        ins
    }
    fn normalize(&mut self){
        let norm=self.norm();
        if norm== 0.0 {
            return;
        }
        self.vector=self.vector.iter().map(|v| v/norm).collect();
    }

    fn norm(&self)->f32 {
        self.vector.iter().map(|v| v*v ).sum::<f32>().sqrt()
    }
}

impl AsRef<[f32]> for Embedding {
    fn as_ref(&self) -> &[f32] {
        &self.vector
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
