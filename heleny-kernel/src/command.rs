use anyhow::Result;
use heleny_bus::Endpoint;
use heleny_proto::message::AnyMessage;
use heleny_service::ServiceHandle;
use tokio::sync::oneshot;

use crate::health::KernelHealth;

pub enum AdminCommand {
    AddService(ServiceHandle),
    DeleteService(&'static str),
    NewEndpoint(&'static str, oneshot::Sender<Endpoint>),
    Shutdown(ShutdownStage),
}

pub enum KernelCommand {
    Shutdown,
    GetHealth(oneshot::Sender<KernelHealth>),
}

pub enum ShutdownStage {
    Start,
    StopAllService,
    StopKernel,
}

pub fn downcast(msg: Box<dyn AnyMessage>) -> Result<Result<Box<AdminCommand>, Box<KernelCommand>>> {
    let command = match msg.as_any().downcast::<AdminCommand>() {
        Ok(command) => return Ok(Ok(command)),
        Err(command) => command,
    };
    match command.downcast::<KernelCommand>() {
        Ok(command) => Ok(Err(command)),
        Err(_) => Err(anyhow::anyhow!("不是 KernelCommand 也不是 AdminCommand")),
    }
}
