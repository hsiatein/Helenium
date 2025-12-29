use std::collections::HashMap;
use anyhow::Context;
use axum::Router;
use axum::extract::State;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::any;
use heleny_proto::frontend_type::FrontendType;
use heleny_proto::message::downcast;
use heleny_proto::name::USER_SERVICE;
use heleny_service::UserServiceMessage;
use heleny_service::get_from_config_service;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use heleny_bus::endpoint::Endpoint;
use heleny_macros::base_service;
use heleny_service::Service;
use heleny_service::WebuiServiceMessage;
use heleny_proto::{message::AnyMessage, role::ServiceRole};
use async_trait::async_trait;
use anyhow::Result;
use heleny_proto::resource::Resource;
use tower_http::services::ServeFile;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::warn;
use uuid::Uuid;
use tower_http::services::ServeDir;
use crate::message::ServiceMessage;
use crate::message::SessionToService;
use crate::register::Register;
use crate::register::SessionEndpoint;
use crate::webui_config::WebuiConfig;
use message::SessionMessage;

mod webui_config;
mod register;
mod message;


#[base_service(deps=["ConfigService","UserService"])]
pub struct WebuiService{
    endpoint:Endpoint,
    router:HashMap<Uuid,mpsc::Sender<ServiceMessage>>,
    app_handle:JoinHandle<()>,
}

#[async_trait]
impl Service for WebuiService {
    type MessageType= WebuiServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let config=get_from_config_service::<WebuiConfig>(&endpoint).await?;
        // 向User服务注册
        endpoint.send(USER_SERVICE, UserServiceMessage::Login(FrontendType::WEB)).await?;
        // 开启 Web 服务
        let serve_dir = ServeDir::new("heleny-webui/dist").not_found_service(ServeFile::new("heleny-webui/dist/index.html"));
        let register=Register::new(endpoint.create_sub_endpoint()?, config.session_buffer);
        let router=Router::new().fallback_service(serve_dir)
        .route("/ws", any(ws_handler)).with_state(register);
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",config.port)).await.context("新建端口监听失败")?;
        info!("正则监听 {} 端口",config.port);
        let app_handle=tokio::spawn(async move{
            if let Err(e) = axum::serve(listener, router).await {
                error!("Axum 服务错误: {}",e);
            };
        });
        open::that(format!("http://127.0.0.1:{}",config.port))?;
        // 新建实例
        let instance=Self {
            endpoint,
            router:HashMap::new(),
            app_handle,
        };
        Ok(Box::new(instance))
    }
    async fn handle(
        &mut self,
        _name: String,
        _role: ServiceRole,
        _msg: WebuiServiceMessage,
    ) -> Result<()>{
        Ok(())
    }
    async fn stop(&mut self){
        self.app_handle.abort();
    }
    async fn handle_sub_endpoint(&mut self, msg: Box<dyn AnyMessage>) -> Result<()>{
        let worker_message = downcast::<SessionToService>(msg)?;
        let SessionToService { token, payload }=worker_message;
        match payload {
            SessionMessage::Register { sender,feedback }=>{
                self.router.insert(token, sender);
                let _=feedback.send(());
                Ok(())
            },
        }
    }
    async fn handle_tick(&mut self, _tick:Instant) -> Result<()>{
        Ok(())
    }
    async fn handle_resource(&mut self, _resource: Resource) -> Result<()> {
        Ok(())
    }
}

impl WebuiService {
    
}

async fn ws_handler(ws: WebSocketUpgrade,State(register): State<Register>) -> Response {
    let endpoint = match register.get_session_endpoint().await {
        Ok(endpoint) => endpoint,
        Err(e) => {
            warn!("新建 ws 握手失败: {}",e);
            return StatusCode::NOT_FOUND.into_response();
        }
    };
    ws.on_upgrade(move |socket| handle_socket(socket,endpoint))
}

async fn handle_socket(mut socket: WebSocket, mut endpoint:SessionEndpoint){
    loop{
        tokio::select! {
            Some(Ok(msg)) = socket.recv()=>{
                if let Err(e) = handle_ws_msg(&mut socket, &endpoint, msg.clone()).await {
                    warn!("处理 msg [{:?}] 失败: {}",msg,e);
                }
            }
            Some(msg) = endpoint.recv()=>{
                if let Err(e) = handle_service_msg(&mut socket, &endpoint, msg.clone()).await {
                    warn!("处理 msg [{:?}] 失败: {}",msg,e);
                }
            }
        }
    }
}

async fn handle_ws_msg(_socket:&mut WebSocket, _endpoint:&SessionEndpoint,msg:Message)->Result<()>{
    debug!("{:?}",msg);
    Ok(())
}

async fn handle_service_msg(socket:&mut WebSocket, _endpoint:&SessionEndpoint,msg:ServiceMessage)->Result<()>{
    let data = serde_json::to_string(&msg)?;
    socket.send(data.into()).await.context("发送 ws 消息失败")
}