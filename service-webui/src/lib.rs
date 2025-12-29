use anyhow::Context;
use axum::Router;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::WebSocket;
use axum::response::Response;
use axum::routing::any;
use heleny_service::get_from_config_service;
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
use tracing::error;

use crate::webui_config::WebuiConfig;

mod webui_config;


#[base_service(deps=["ConfigService"])]
pub struct WebuiService{
    endpoint:Endpoint,
    app_handle:JoinHandle<()>,
}

#[derive(Debug)]
enum _WorkerMessage{
    
}

#[async_trait]
impl Service for WebuiService {
    type MessageType= WebuiServiceMessage;
    async fn new(endpoint: Endpoint) -> Result<Box<Self>>{
        let router:Router<()>=Router::new().route("/ws", any(handler));
        let config=get_from_config_service::<WebuiConfig>(&endpoint).await?;
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",config.port)).await.context("新建端口监听失败")?;
        let app_handle=tokio::spawn(async move{
            if let Err(e) = axum::serve(listener, router).await {
                error!("Axum 服务错误: {}",e);
            };
        });
        let instance=Self {
            endpoint,
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

    }
    async fn handle_sub_endpoint(&mut self, _msg: Box<dyn AnyMessage>) -> Result<()>{
        Ok(())
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

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket){

}