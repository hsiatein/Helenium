use crate::config::WebuiConfig;
use crate::message::SessionToService;
use crate::register::Register;
use crate::register::SessionEndpoint;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use axum::Router;
use axum::extract::State;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header::CACHE_CONTROL;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::any;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_proto::AnyMessage;
use heleny_proto::FrontendMessage;
use heleny_proto::FrontendType;
use heleny_proto::Resource;
use heleny_proto::ServiceRole;
use heleny_proto::USER_SERVICE;
use heleny_proto::downcast;
use heleny_service::Service;
use heleny_service::UserServiceMessage;
use heleny_service::WebuiServiceMessage;
use heleny_service::get_from_config_service;
use message::SessionMessage;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceBuilder;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

mod command;
mod config;
mod message;
mod register;

#[base_service(deps=["ConfigService","UserService"])]
pub struct WebuiService {
    endpoint: Endpoint,
    router: HashMap<Uuid, mpsc::Sender<FrontendMessage>>,
    app_handle: JoinHandle<()>,
    /// 第一层session id，第二层task id
    session_task_logs: HashMap<Uuid, HashMap<Uuid, JoinHandle<()>>>,
}

#[async_trait]
impl Service for WebuiService {
    type MessageType = WebuiServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>> {
        let config = get_from_config_service::<WebuiConfig>(&endpoint).await?;
        // 向User服务注册
        endpoint
            .send(USER_SERVICE, UserServiceMessage::Login(FrontendType::WEB))
            .await?;
        // 开启 Web 服务
        let serve_dir = ServeDir::new("heleny-webui/dist")
            .not_found_service(ServeFile::new("heleny-webui/dist/index.html"));
        let serve_dir = ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::overriding(
                CACHE_CONTROL,
                HeaderValue::from_static("no-store, must-revalidate"),
            ))
            .service(serve_dir);
        let register = Register::new(endpoint.create_sub_endpoint()?, config.session_buffer);
        let router = Router::new()
            .fallback_service(serve_dir)
            .route("/ws", any(ws_handler))
            .with_state(register);
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .context("新建端口监听失败")?;
        info!("正则监听 {} 端口", config.port);
        let app_handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, router).await {
                error!("Axum 服务错误: {}", e);
            };
        });
        let cache_bust = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        open::that(format!("http://127.0.0.1:{}/?v={}", config.port, cache_bust))?;
        // 新建实例
        let instance = Self {
            endpoint,
            router: HashMap::new(),
            app_handle,
            session_task_logs: HashMap::new(),
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        msg: WebuiServiceMessage,
    ) -> Result<()> {
        match msg {
            WebuiServiceMessage::UserDecision(user_decision) => {
                self.send_to_all_sessions(FrontendMessage::UserDecision(user_decision))
                    .await
            }

            WebuiServiceMessage::SendToFrontend { session, message } => {
                self.send_to_session(session, message).await
            }
        }
    }
    async fn stop(&mut self) {
        self.app_handle.abort();
        let _ = self
            .endpoint
            .send(USER_SERVICE, UserServiceMessage::Logout)
            .await;
    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()> {
        let worker_message = downcast::<SessionToService>(msg)?;
        let SessionToService { token, payload } = worker_message;
        match payload {
            SessionMessage::Register { sender, feedback } => {
                self.router.insert(token, sender);
                self.session_task_logs.insert(token, HashMap::new());
                let _ = feedback.send(());
                Ok(())
            }
            SessionMessage::Logout => {
                self.router.remove(&token);
                self.session_task_logs.remove(&token);
                Ok(())
            }
            SessionMessage::UserInput { input } => {
                let command = serde_json::from_str(&input)?;
                self.handle_command(token, command).await
            }
        }
    }

    async fn handle_tick(&mut self, _tick: Instant) -> Result<()> {
        Ok(())
    }
    async fn handle_resource(&mut self, resource: Resource) -> Result<()> {
        self.send_to_all_sessions(FrontendMessage::UpdateResource(resource))
            .await
    }
}

impl WebuiService {
    async fn send_to_all_sessions(&self, msg: FrontendMessage) -> Result<()> {
        for (_token, tx) in &self.router {
            if let Err(e) = tx.send(msg.clone()).await {
                warn!("发给所有 User 失败: {}", e)
            };
        }
        Ok(())
    }
    async fn send_to_session(&self, session: Uuid, msg: FrontendMessage) -> Result<()> {
        let _ = &self
            .router
            .iter()
            .find(|(id, _tx)| **id == session)
            .context("没找到对应 Session")?
            .1
            .send(msg)
            .await?;
        Ok(())
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(register): State<Register>) -> Response {
    let endpoint = match register.get_session_endpoint().await {
        Ok(endpoint) => endpoint,
        Err(e) => {
            warn!("新建 ws 握手失败: {}", e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket, endpoint))
}

async fn handle_socket(mut socket: WebSocket, mut endpoint: SessionEndpoint) {
    info!("新的前端连接已建立 WebSocket 会话");
    loop {
        tokio::select! {
            res = socket.recv() => {
                match res {
                    Some(Ok(msg)) => {
                        if let Err(e) = handle_ws_msg(&mut socket, &endpoint, msg).await {
                            warn!("处理消息失败: {}", e);
                        }
                    }
                    Some(Err(e)) => {
                        debug!("WebSocket 出错: {}", e);
                        break;
                    }
                    None => {
                        info!("前端已断开连接，正在关闭 session");
                        break;
                    }
                }
            }
            msg = endpoint.recv() => {
                match msg {
                    Some(msg) => {
                        if let Err(e) = handle_service_msg(&mut socket, &endpoint, msg).await {
                            warn!("处理消息失败: {}", e);
                        }
                    }
                    None => {
                        debug!("服务层 endpoint 已关闭");
                        break;
                    }
                }
            }
        }
    }
    let _ = endpoint.send(SessionMessage::Logout).await;
}

async fn handle_ws_msg(
    _socket: &mut WebSocket,
    endpoint: &SessionEndpoint,
    msg: Message,
) -> Result<()> {
    let msg = msg.into_text()?;
    let msg = String::from_utf8_lossy(msg.as_bytes()).to_string();
    if msg.is_empty() {
        return Ok(());
    }
    endpoint
        .send(SessionMessage::UserInput { input: msg })
        .await?;
    Ok(())
}

async fn handle_service_msg(
    socket: &mut WebSocket,
    _endpoint: &SessionEndpoint,
    msg: FrontendMessage,
) -> Result<()> {
    let data = serde_json::to_string(&msg)?;
    socket.send(data.into()).await.context("发送 ws 消息失败")
}
