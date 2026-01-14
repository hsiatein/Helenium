use anyhow::Context;
use anyhow::Result;
use heleny_bus::endpoint::SubEndpoint;
use heleny_proto::FrontendMessage;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::message::SessionMessage;
use crate::message::SessionToService;

#[derive(Clone)]
pub struct Register {
    to_service: SubEndpoint,
    buffer: usize,
}

impl Register {
    pub fn new(to_service: SubEndpoint, buffer: usize) -> Self {
        Self { to_service, buffer }
    }

    pub async fn get_session_endpoint(&self) -> Result<SessionEndpoint> {
        let (tx, rx) = mpsc::channel(self.buffer);
        let token = Uuid::new_v4();
        let (feedback_tx, feedback_rx) = oneshot::channel();
        let endpoint = SessionEndpoint::new(token, self.to_service.clone(), rx);
        endpoint
            .send(SessionMessage::Register {
                sender: tx,
                feedback: feedback_tx,
            })
            .await
            .context("发送注册消息失败")?;
        let _ = feedback_rx.await.context("获取注册反馈失败")?;
        Ok(endpoint)
    }
}

pub struct SessionEndpoint {
    token: Uuid,
    to_service: SubEndpoint,
    from_service: mpsc::Receiver<FrontendMessage>,
}

impl SessionEndpoint {
    pub fn new(
        token: Uuid,
        to_service: SubEndpoint,
        from_service: mpsc::Receiver<FrontendMessage>,
    ) -> Self {
        Self {
            token,
            to_service,
            from_service,
        }
    }

    pub async fn send(&self, msg: SessionMessage) -> Result<()> {
        self.to_service
            .send(Box::new(SessionToService {
                token: self.token,
                payload: msg,
            }))
            .await
            .context("Session 发送消息给 Webui Service 失败")
    }

    pub async fn recv(&mut self) -> Option<FrontendMessage> {
        self.from_service.recv().await
    }
}
