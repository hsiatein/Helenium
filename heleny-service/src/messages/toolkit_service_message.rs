use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use heleny_bus::endpoint::Endpoint;
use heleny_proto::CanRequestConsent;
use heleny_proto::ConsentRequestion;
use heleny_proto::HelenyTool;
use heleny_proto::HelenyToolFactory;
use heleny_proto::ToolIntent;
use heleny_proto::USER_SERVICE;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::UserServiceMessage;

#[derive(Debug)]
pub enum ToolkitServiceMessage {
    GetIntro {
        feedback: oneshot::Sender<String>,
    },
    GetToolkit {
        tool_names: Vec<String>,
        task_id: Uuid,
        task_description: String,
        feedback: oneshot::Sender<Toolkit>,
    },
    Register {
        factory: Box<dyn HelenyToolFactory>,
    },
    Reload,
    EnableTool {
        name: String,
        enable: bool,
    }
}

#[derive(Debug)]
pub struct Toolkit {
    endpoint: ToolkitEndpoint,
    tool_manuals: String,
    tools: HashMap<String, Box<dyn HelenyTool>>,
}

impl Toolkit {
    pub fn new(
        task_id: Uuid,
        task_description: String,
        endpoint: Endpoint,
        tool_manuals: String,
        tools: HashMap<String, Box<dyn HelenyTool>>,
    ) -> Self {
        let endpoint = ToolkitEndpoint::new(task_id, task_description, endpoint);
        Toolkit {
            endpoint,
            tool_manuals,
            tools,
        }
    }
    pub async fn invoke(&mut self, intent: ToolIntent) -> String {
        let ToolIntent {
            reason,
            tool,
            command,
            args,
        } = intent;
        let (Some(tool_name), Some(command)) = (tool, command) else {
            return "你没有提供 command 字段! 注意, 你要把工具名写在tool字段, 命令名写在command字段, 参数写在args字段, 绝对不能把调用放在一个字段里! 你不能在tool字段里嵌套json放command和args字段!!!".to_string();
        };
        match self.tools.get_mut(&tool_name) {
            Some(tool) => {
                self.endpoint.set_reason(reason);
                match tool.invoke(command, args, Box::new(&self.endpoint)).await {
                    Ok(result) => result,
                    Err(e) => format!("工具调用失败: {}", e),
                }
            }
            None => format!("未找到工具: {}", tool_name),
        }
    }

    pub fn get_manuals(&self) -> &str {
        &self.tool_manuals
    }
}

#[derive(Debug)]
pub struct ToolkitEndpoint {
    task_id: Uuid,
    task_description: String,
    endpoint: Endpoint,
    reason: String,
}

impl ToolkitEndpoint {
    pub fn new(task_id: Uuid, task_description: String, endpoint: Endpoint) -> Self {
        ToolkitEndpoint {
            task_id,
            task_description,
            endpoint,
            reason: String::new(),
        }
    }

    pub fn set_reason(&mut self, reason: String) {
        self.reason = reason;
    }
}

#[async_trait]
impl CanRequestConsent for ToolkitEndpoint {
    async fn request_consent(&self, description: String) -> Result<()> {
        let (feedback_sender, feedback_receiver) = oneshot::channel();
        let requestion = ConsentRequestion {
            task_id: self.task_id,
            task_description: self.task_description.clone(),
            reason: self.reason.clone(),
            description,
            feedback: feedback_sender,
        };
        self.endpoint
            .send(
                USER_SERVICE,
                UserServiceMessage::RequestConsent { body: requestion },
            )
            .await
            .context("发起申请失败")?;
        let feedback = feedback_receiver.await.context("等待用户反馈失败")?;
        if feedback {
            Ok(())
        } else {
            Err(anyhow::anyhow!("用户拒绝了工具调用"))
        }
    }
}
