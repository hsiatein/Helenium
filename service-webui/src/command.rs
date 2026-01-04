use anyhow::Result;
use heleny_proto::{FrontendMessage, name::{HUB_SERVICE, KERNEL_NAME, MEMORY_SERVICE}, resource::{DISPLAY_MESSAGES, HEALTH, Resource, ResourcePayload}};
use heleny_service::{HubServiceMessage, KernelMessage, MemoryServiceMessage};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{WebuiService};

impl WebuiService {
    pub async fn handle_command(&mut self, token:Uuid, command:String)->Result<()>{
        let mut args=command.split_whitespace();
        let arg0=args.next();
        if arg0==Some("get_history") {
            if let Some(arg)=args.next() {
                let id_upper_bound:i64=arg.parse()?;
                let (tx,rx)=oneshot::channel();
                self.endpoint.send(MEMORY_SERVICE, MemoryServiceMessage::Get { id_upper_bound, feedback: tx }).await?;
                let history=rx.await?;
                self.send_to_session(token, FrontendMessage::UpdateResource(Resource { name: DISPLAY_MESSAGES.to_string(), payload: ResourcePayload::DisplayMessages{new:false,messages:history}})).await?;
            }
        }
        else if arg0==Some("shutdown") {
            self.endpoint.send(KERNEL_NAME, KernelMessage::Shutdown).await?;
        }
        else if arg0==Some("get_health") {
            let (tx,rx)=oneshot::channel();
            self.endpoint.send(HUB_SERVICE, HubServiceMessage::Get { resource_name: HEALTH.to_string(), feedback: tx }).await?;
            let health=rx.await?;
            self.send_to_session(token, FrontendMessage::UpdateResource(Resource { name: HEALTH.to_string(), payload: health})).await?;
        }
        Ok(())
    }
}